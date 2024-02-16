use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::Hasher;

use crate::configuration::TestConfiguration;
use crate::connector::Connector;
use crate::reporter::{Reporter, ReporterExt};
use crate::results::TestResults;
use crate::snapshot::snapshot_test;

use super::error::Error;
use indexmap::IndexMap;

use ndc_client::models::{self, OrderDirection};
use rand::rngs::SmallRng;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::Rng;

pub async fn run_all_tests<C: Connector, R: Reporter>(
    configuration: &TestConfiguration,
    connector: &C,
    reporter: &R,
    rng: &mut SmallRng,
    results: &RefCell<TestResults>,
) -> Option<()> {
    let capabilities = reporter
        .nest("Capabilities", results, async {
            let capabilities = reporter
                .test("Fetching /capabilities", results, async {
                    let response = connector.get_capabilities().await?;
                    for snapshots_dir in configuration.snapshots_dir.iter() {
                        snapshot_test(snapshots_dir.join("capabilities").as_path(), &response)?;
                    }
                    Ok(response)
                })
                .await?;

            let _ = reporter
                .test("Validating capabilities", results, async {
                    validate_capabilities(&capabilities)
                })
                .await;

            Some(capabilities)
        })
        .await?;

    let schema = reporter
        .nest("Schema", results, async {
            let schema = reporter
                .test("Fetching schema", results, async {
                    let response = connector.get_schema().await?;
                    for snapshots_dir in configuration.snapshots_dir.iter() {
                        snapshot_test(snapshots_dir.join("schema").as_path(), &response)?;
                    }
                    Ok(response)
                })
                .await?;

            reporter
                .nest("Validating schema", results, async {
                    validate_schema(reporter, &schema, results).await
                })
                .await?;

            Some(schema)
        })
        .await?;

    reporter
        .nest(
            "Query",
            results,
            test_query(
                configuration,
                connector,
                reporter,
                &capabilities,
                &schema,
                rng,
                results,
            ),
        )
        .await;

    Some(())
}

pub fn validate_capabilities(capabilities: &models::CapabilitiesResponse) -> Result<(), Error> {
    let pkg_version = env!("CARGO_PKG_VERSION");
    let spec_version = semver::VersionReq::parse(format!("^{}", pkg_version).as_str())?;
    let claimed_version = semver::Version::parse(capabilities.version.as_str())?;
    if !spec_version.matches(&claimed_version) {
        return Err(Error::IncompatibleSpecification(
            claimed_version,
            spec_version,
        ));
    }

    Ok(())
}

pub async fn validate_schema<R: Reporter>(
    reporter: &R,
    schema: &models::SchemaResponse,
    results: &RefCell<TestResults>,
) -> Option<()> {
    let _ = reporter
        .test("object_types", results, async {
            for (_type_name, object_type) in schema.object_types.iter() {
                for (_field_name, object_field) in object_type.fields.iter() {
                    validate_type(schema, &object_field.r#type)?;
                }
            }
            Ok(())
        })
        .await;

    reporter
        .nest("Collections", results, async {
            for collection_info in schema.collections.iter() {
                reporter
                    .nest(collection_info.name.as_str(), results, async {
                        let _ = reporter
                            .test("Arguments", results, async {
                                for (_arg_name, arg_info) in collection_info.arguments.iter() {
                                    validate_type(schema, &arg_info.argument_type)?;
                                }
                                Ok(())
                            })
                            .await;

                        let _ = reporter
                            .test("Collection type", results, async {
                                let _ = schema
                                    .object_types
                                    .get(collection_info.collection_type.as_str())
                                    .ok_or(Error::CollectionTypeIsNotDefined(
                                        collection_info.collection_type.clone(),
                                    ))?;

                                Ok(())
                            })
                            .await;
                    })
                    .await;
            }
        })
        .await;

    reporter
        .nest("Functions", results, async {
            for function_info in schema.functions.iter() {
                reporter
                    .nest(function_info.name.as_str(), results, async {
                        let _ = reporter
                            .test("Result type", results, async {
                                validate_type(schema, &function_info.result_type)
                            })
                            .await;

                        let _ = reporter
                            .test("Arguments", results, async {
                                for (_arg_name, arg_info) in function_info.arguments.iter() {
                                    validate_type(schema, &arg_info.argument_type)?;
                                }

                                Ok(())
                            })
                            .await;
                    })
                    .await;
            }

            reporter
                .nest("Procedures", results, async {
                    for procedure_info in schema.procedures.iter() {
                        reporter
                            .nest(procedure_info.name.as_str(), results, async {
                                let _ = reporter
                                    .test("Result type", results, async {
                                        validate_type(schema, &procedure_info.result_type)
                                    })
                                    .await;

                                let _ = reporter
                                    .test("Arguments", results, async {
                                        for (_arg_name, arg_info) in procedure_info.arguments.iter()
                                        {
                                            validate_type(schema, &arg_info.argument_type)?;
                                        }

                                        Ok(())
                                    })
                                    .await;
                            })
                            .await;
                    }
                })
                .await;
        })
        .await;

    Some(())
}

pub fn validate_type(schema: &models::SchemaResponse, r#type: &models::Type) -> Result<(), Error> {
    match r#type {
        models::Type::Named { name } => {
            if !schema.object_types.contains_key(name.as_str())
                && !schema.scalar_types.contains_key(name.as_str())
            {
                return Err(Error::NamedTypeIsNotDefined(name.clone()));
            }
        }
        models::Type::Array { element_type } => {
            validate_type(schema, element_type)?;
        }
        models::Type::Nullable { underlying_type } => {
            validate_type(schema, underlying_type)?;
        }
        models::Type::Predicate { object_type_name } => {
            if !schema.object_types.contains_key(object_type_name.as_str()) {
                return Err(Error::ObjectTypeIsNotDefined(object_type_name.clone()));
            }
        }
    }

    Ok(())
}

pub async fn test_query<C: Connector, R: Reporter>(
    configuration: &TestConfiguration,
    connector: &C,
    reporter: &R,
    capabilities: &models::CapabilitiesResponse,
    schema: &models::SchemaResponse,
    rng: &mut SmallRng,
    results: &RefCell<TestResults>,
) {
    for collection_info in schema.collections.iter() {
        reporter
            .nest(collection_info.name.as_str(), results, async {
                if collection_info.arguments.is_empty() {
                    reporter
                        .nest("Simple queries", results, async {
                            test_simple_queries(
                                configuration,
                                connector,
                                reporter,
                                rng,
                                results,
                                schema,
                                collection_info,
                            )
                            .await
                        })
                        .await;

                    if capabilities.capabilities.relationships.is_some() {
                        reporter
                            .nest("Relationship queries", results, async {
                                test_relationship_queries(
                                    configuration,
                                    connector,
                                    reporter,
                                    results,
                                    schema,
                                    collection_info,
                                )
                                .await
                            })
                            .await;
                    }

                    reporter
                        .nest("Aggregate queries", results, async {
                            test_aggregate_queries(
                                configuration,
                                connector,
                                reporter,
                                schema,
                                collection_info,
                                results,
                            )
                            .await
                        })
                        .await;
                } else {
                    eprintln!("Skipping parameterized collection {}", collection_info.name);
                }
            })
            .await;
    }
}

async fn test_simple_queries<C: Connector, R: Reporter>(
    configuration: &TestConfiguration,
    connector: &C,
    reporter: &R,
    rng: &mut SmallRng,
    results: &RefCell<TestResults>,
    schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
) -> Option<()> {
    let collection_type = schema
        .object_types
        .get(collection_info.collection_type.as_str())?;

    let rows = reporter
        .test(
            "Select top N",
            results,
            test_select_top_n_rows(configuration, connector, collection_type, collection_info),
        )
        .await?;

    let context = make_context(collection_type, rows).ok()?;

    reporter
        .test("Predicates", results, async {
            if let Some(context) = context {
                for _ in 0..10 {
                    if let Some(predicate) = make_predicate(schema, &context, rng)? {
                        test_select_top_n_rows_with_predicate(
                            configuration,
                            connector,
                            &predicate,
                            collection_type,
                            collection_info,
                        )
                        .await?;
                    }
                }
            } else {
                eprintln!("Skipping empty collection {}", collection_info.name);
            }

            Ok(())
        })
        .await?;

    reporter
        .test("Sorting", results, async {
            for _ in 0..10 {
                if let Some(order_by_element) =
                    make_order_by_element(collection_type.clone(), schema, rng)
                {
                    test_select_top_n_rows_with_sort(
                        configuration,
                        connector,
                        vec![order_by_element], // TODO
                        collection_type,
                        collection_info,
                    )
                    .await?;
                } else {
                    eprintln!("Skipping empty collection {}", collection_info.name);
                }
            }

            Ok(())
        })
        .await
}

async fn test_select_top_n_rows<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
) -> Result<Vec<IndexMap<String, models::RowFieldValue>>, Error> {
    let fields = select_all_columns(collection_type);
    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: None,
            fields: Some(fields.clone()),
            limit: Some(10),
            offset: None,
            order_by: None,
            predicate: None,
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };

    let response = execute_and_snapshot_query(configuration, connector, query_request).await?;

    expect_single_rows(&response)
}

async fn test_select_top_n_rows_with_predicate<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    predicate: &GeneratedExpression,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
) -> Result<ndc_client::models::QueryResponse, Error> {
    let fields = select_all_columns(collection_type);

    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: None,
            fields: Some(fields),
            limit: Some(10),
            offset: None,
            order_by: None,
            predicate: Some(predicate.expr.clone()),
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };

    let response = execute_and_snapshot_query(configuration, connector, query_request).await?;

    if predicate.expect_nonempty {
        expect_single_non_empty_rows(&response)?;
    }

    Ok(response)
}

async fn test_select_top_n_rows_with_sort<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    elements: Vec<models::OrderByElement>,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
) -> Result<ndc_client::models::QueryResponse, Error> {
    let fields = select_all_columns(collection_type);

    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: None,
            fields: Some(fields),
            limit: Some(10),
            offset: None,
            order_by: Some(models::OrderBy { elements }),
            predicate: None,
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };

    let response = execute_and_snapshot_query(configuration, connector, query_request).await?;

    expect_single_rows(&response)?;

    Ok(response)
}

async fn test_relationship_queries<C: Connector, R: Reporter>(
    configuration: &TestConfiguration,
    connector: &C,
    reporter: &R,
    results: &RefCell<TestResults>,
    schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
) -> Option<()> {
    let collection_type = schema
        .object_types
        .get(collection_info.collection_type.as_str())
        .ok_or(Error::CollectionTypeIsNotDefined(
            collection_info.collection_type.clone(),
        ))
        .ok()?;

    for (foreign_key_name, foreign_key) in collection_info.foreign_keys.iter() {
        reporter
            .nest(foreign_key_name, results, async {
                let _ = reporter
                    .test(
                        "Object relationship",
                        results,
                        select_top_n_using_foreign_key(
                            configuration,
                            connector,
                            collection_type,
                            collection_info,
                            schema,
                            foreign_key_name,
                            foreign_key,
                        ),
                    )
                    .await;

                let _ = reporter
                    .test(
                        "Array relationship",
                        results,
                        select_top_n_using_foreign_key_as_array_relationship(
                            configuration,
                            connector,
                            collection_type,
                            collection_info,
                            schema,
                            foreign_key_name,
                            foreign_key,
                        ),
                    )
                    .await;

                Some(())
            })
            .await;
    }

    Some(())
}

async fn select_top_n_using_foreign_key<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
    schema: &models::SchemaResponse,
    foreign_key_name: &str,
    foreign_key: &models::ForeignKeyConstraint,
) -> Result<(), Error> {
    let mut fields = select_all_columns(collection_type);

    let other_collection = schema
        .collections
        .iter()
        .find(|c| c.name == foreign_key.foreign_collection)
        .ok_or(Error::CollectionIsNotDefined(
            foreign_key.foreign_collection.clone(),
        ))?;

    if other_collection.arguments.is_empty() {
        let other_collection_type = schema
            .object_types
            .get(other_collection.collection_type.as_str())
            .ok_or(Error::CollectionTypeIsNotDefined(
                other_collection.collection_type.clone(),
            ))?;

        let other_fields = select_all_columns(other_collection_type);

        fields.insert(
            foreign_key_name.into(),
            models::Field::Relationship {
                query: Box::new(models::Query {
                    aggregates: None,
                    fields: Some(other_fields.clone()),
                    limit: Some(10),
                    offset: None,
                    order_by: None,
                    predicate: None,
                }),
                relationship: "__relationship".into(),
                arguments: BTreeMap::new(),
            },
        );

        let query_request = models::QueryRequest {
            collection: collection_info.name.clone(),
            query: models::Query {
                aggregates: None,
                fields: Some(fields.clone()),
                limit: Some(10),
                offset: None,
                order_by: None,
                predicate: None,
            },
            arguments: BTreeMap::new(),
            collection_relationships: BTreeMap::from_iter([(
                "__relationship".into(),
                models::Relationship {
                    column_mapping: foreign_key.column_mapping.clone(),
                    relationship_type: models::RelationshipType::Object,
                    target_collection: foreign_key.foreign_collection.clone(),
                    arguments: BTreeMap::new(),
                },
            )]),
            variables: None,
        };

        let response = execute_and_snapshot_query(configuration, connector, query_request).await?;

        expect_single_rows(&response)?;
    } else {
        eprintln!("Skipping parameterized relationship {}", foreign_key_name);
    }

    Ok(())
}

async fn select_top_n_using_foreign_key_as_array_relationship<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
    schema: &models::SchemaResponse,
    foreign_key_name: &str,
    foreign_key: &models::ForeignKeyConstraint,
) -> Result<(), Error> {
    let fields = select_all_columns(collection_type);

    let other_collection = schema
        .collections
        .iter()
        .find(|c| c.name == foreign_key.foreign_collection)
        .ok_or(Error::CollectionIsNotDefined(
            foreign_key.foreign_collection.clone(),
        ))?;

    if other_collection.arguments.is_empty() {
        let other_collection_type = schema
            .object_types
            .get(other_collection.collection_type.as_str())
            .ok_or(Error::CollectionTypeIsNotDefined(
                other_collection.collection_type.clone(),
            ))?;

        let mut other_fields = select_all_columns(other_collection_type);

        other_fields.insert(
            foreign_key_name.into(),
            models::Field::Relationship {
                query: Box::new(models::Query {
                    aggregates: None,
                    fields: Some(fields.clone()),
                    limit: Some(10),
                    offset: None,
                    order_by: None,
                    predicate: None,
                }),
                relationship: "__array_relationship".into(),
                arguments: BTreeMap::new(),
            },
        );

        let mut column_mapping = BTreeMap::new();

        for (column, other_column) in foreign_key.column_mapping.iter() {
            column_mapping.insert(other_column.clone(), column.clone());
        }

        let query_request = models::QueryRequest {
            collection: foreign_key.foreign_collection.clone(),
            query: models::Query {
                aggregates: None,
                fields: Some(other_fields.clone()),
                limit: Some(10),
                offset: None,
                order_by: None,
                predicate: None,
            },
            arguments: BTreeMap::new(),
            collection_relationships: BTreeMap::from_iter([(
                "__array_relationship".into(),
                models::Relationship {
                    column_mapping,
                    relationship_type: models::RelationshipType::Array,
                    target_collection: collection_info.name.clone(),
                    arguments: BTreeMap::new(),
                },
            )]),
            variables: None,
        };

        let response = execute_and_snapshot_query(configuration, connector, query_request).await?;

        expect_single_rows(&response)?;
    } else {
        eprintln!("Skipping parameterized relationship {}", foreign_key_name);
    }

    Ok(())
}

async fn execute_and_snapshot_query<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    query_request: models::QueryRequest,
) -> Result<models::QueryResponse, Error> {
    use std::hash::Hash;

    let request_json = serde_json::to_string_pretty(&query_request).map_err(Error::SerdeError)?;
    let response = connector.query(query_request).await?;

    if let Some(snapshots_dir) = &configuration.snapshots_dir {
        let mut hasher = DefaultHasher::new();
        request_json.hash(&mut hasher);
        let hash = hasher.finish();

        let snapshot_subdir = {
            let mut builder = snapshots_dir.clone();
            builder.extend(vec!["query", format!("{:x}", hash).as_str()]);
            builder
        };

        snapshot_test(snapshot_subdir.join("expected.json").as_path(), &response)?;

        std::fs::write(snapshot_subdir.join("request.json").as_path(), request_json)
            .map_err(Error::CannotOpenSnapshotFile)?;
    }

    Ok(response)
}

fn expect_single_non_empty_rows(
    response: &models::QueryResponse,
) -> Result<Vec<IndexMap<String, models::RowFieldValue>>, Error> {
    let rows = expect_single_rows(response)?;

    if rows.is_empty() {
        return Err(Error::ExpectedNonEmptyRows);
    }

    Ok(rows)
}

fn expect_single_rows(
    response: &models::QueryResponse,
) -> Result<Vec<IndexMap<String, models::RowFieldValue>>, Error> {
    if response.0.len() != 1 {
        return Err(Error::ExpectedSingleRowSet);
    }

    let row_set = &response.0[0];
    let rows = row_set
        .rows
        .clone()
        .ok_or(Error::RowsShouldBeNonNullInRowSet)?;

    Ok(rows)
}

#[derive(Clone, Debug)]
struct GeneratedValue {
    field_name: String,
    value: serde_json::Value,
}

#[derive(Clone, Debug)]
struct Context<'a> {
    collection_type: &'a models::ObjectType,
    values: Vec<GeneratedValue>,
}

fn make_context(
    collection_type: &models::ObjectType,
    rows: Vec<IndexMap<String, models::RowFieldValue>>,
) -> Result<Option<Context>, Error> {
    let mut values = vec![];

    for row in rows {
        for (field_name, _) in collection_type.fields.iter() {
            if !row.contains_key(field_name.as_str()) {
                return Err(Error::MissingField(field_name.clone()));
            }
        }

        for (field_name, field_value) in row {
            values.push(GeneratedValue {
                field_name,
                value: field_value.0,
            });
        }
    }

    Ok(if values.is_empty() {
        None
    } else {
        Some(Context {
            collection_type,
            values,
        })
    })
}

fn make_value<'a>(context: &'a Context, rng: &mut SmallRng) -> Result<&'a GeneratedValue, Error> {
    context
        .values
        .choose(rng)
        .ok_or(Error::ExpectedNonEmptyRows)
}

#[derive(Clone, Debug)]
struct GeneratedExpression {
    expr: models::Expression,
    expect_nonempty: bool,
}

fn make_predicate(
    schema: &models::SchemaResponse,
    context: &Context,
    rng: &mut SmallRng,
) -> Result<Option<GeneratedExpression>, Error> {
    let value = make_value(context, rng)?;
    let field_type = &context
        .collection_type
        .fields
        .get(value.field_name.as_str())
        .ok_or(Error::UnexpectedField(value.field_name.clone()))?
        .r#type;

    let mut expressions: Vec<GeneratedExpression> = vec![];

    if is_nullable_type(field_type) {
        expressions.push(GeneratedExpression {
            expr: models::Expression::UnaryComparisonOperator {
                column: models::ComparisonTarget::Column {
                    name: value.field_name.clone(),
                    path: vec![],
                },
                operator: models::UnaryComparisonOperator::IsNull,
            },
            expect_nonempty: false,
        });
    }

    if let Some(field_type_name) = get_named_type(field_type) {
        if let Some(field_scalar_type) = schema.scalar_types.get(field_type_name.as_str()) {
            for (operator_name, operator) in field_scalar_type.comparison_operators.iter() {
                match operator {
                    models::ComparisonOperatorDefinition::Equal => {
                        expressions.push(GeneratedExpression {
                            expr: models::Expression::BinaryComparisonOperator {
                                column: models::ComparisonTarget::Column {
                                    name: value.field_name.clone(),
                                    path: vec![],
                                },
                                operator: operator_name.clone(),
                                value: models::ComparisonValue::Scalar {
                                    value: value.value.clone(),
                                },
                            },
                            expect_nonempty: true,
                        });
                    }
                    models::ComparisonOperatorDefinition::In => {
                        expressions.push(GeneratedExpression {
                            expr: models::Expression::BinaryComparisonOperator {
                                column: models::ComparisonTarget::Column {
                                    name: value.field_name.clone(),
                                    path: vec![],
                                },
                                operator: operator_name.clone(),
                                value: models::ComparisonValue::Scalar {
                                    value: serde_json::Value::Array(vec![value.value.clone()]), // TODO
                                },
                            },
                            expect_nonempty: true,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(if expressions.is_empty() {
        None
    } else {
        expressions.choose(rng).cloned()
    })
}

fn is_nullable_type(ty: &models::Type) -> bool {
    match ty {
        models::Type::Named { name: _ } => false,
        models::Type::Nullable { underlying_type: _ } => true,
        models::Type::Array { element_type: _ } => false,
        models::Type::Predicate {
            object_type_name: _,
        } => false,
    }
}

fn as_named_type(ty: &models::Type) -> Option<&String> {
    match ty {
        models::Type::Named { name } => Some(name),
        models::Type::Nullable { underlying_type } => as_named_type(underlying_type),
        models::Type::Array { element_type: _ } => None,
        models::Type::Predicate {
            object_type_name: _,
        } => None,
    }
}

fn get_named_type(ty: &models::Type) -> Option<&String> {
    match ty {
        models::Type::Named { name } => Some(name),
        models::Type::Nullable { underlying_type } => get_named_type(underlying_type),
        models::Type::Array { element_type: _ } => None,
        models::Type::Predicate {
            object_type_name: _,
        } => None,
    }
}

fn make_order_by_element(
    collection_type: models::ObjectType,
    schema: &models::SchemaResponse,
    rng: &mut SmallRng,
) -> Option<models::OrderByElement> {
    let mut sortable_fields = BTreeMap::new();

    for (field_name, field) in collection_type.fields.into_iter() {
        if let Some(name) = as_named_type(&field.r#type) {
            if schema.scalar_types.contains_key(name) {
                sortable_fields.insert(field_name, field);
            }
        }
    }

    if sortable_fields.is_empty() {
        None
    } else {
        let (field_name, _) = sortable_fields.iter().choose(rng)?;

        let order_direction = if rng.gen_bool(0.5) {
            OrderDirection::Asc
        } else {
            OrderDirection::Desc
        };

        Some(models::OrderByElement {
            order_direction,
            target: models::OrderByTarget::Column {
                name: field_name.clone(),
                path: vec![],
            },
        })
    }
}

fn select_all_columns(collection_type: &models::ObjectType) -> IndexMap<String, models::Field> {
    collection_type
        .fields
        .iter()
        .map(|f| {
            (
                f.0.clone(),
                models::Field::Column {
                    column: f.0.clone(),
                    fields: None,
                },
            )
        })
        .collect::<IndexMap<String, models::Field>>()
}

async fn test_aggregate_queries<C: Connector, R: Reporter>(
    configuration: &TestConfiguration,
    connector: &C,
    reporter: &R,
    schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
    results: &RefCell<TestResults>,
) -> Option<()> {
    let collection_type = schema
        .object_types
        .get(collection_info.collection_type.as_str())
        .ok_or(Error::CollectionTypeIsNotDefined(
            collection_info.collection_type.clone(),
        ))
        .ok()?;

    let total_count = reporter
        .test("star_count", results, async {
            test_star_count_aggregate(configuration, connector, collection_info).await
        })
        .await?;

    let _ = reporter
        .test("column_count", results, async {
            test_column_count_aggregate(
                configuration,
                connector,
                collection_info,
                collection_type,
                total_count,
            )
            .await
        })
        .await;

    Some(())
}

async fn test_star_count_aggregate<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    collection_info: &models::CollectionInfo,
) -> Result<u64, Error> {
    let aggregates = IndexMap::from([("count".into(), models::Aggregate::StarCount {})]);
    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: Some(aggregates),
            fields: None,
            limit: Some(10),
            offset: None,
            order_by: None,
            predicate: None,
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };
    let response = execute_and_snapshot_query(configuration, connector, query_request).await?;
    if let [row_set] = &*response.0 {
        if row_set.rows.is_some() {
            return Err(Error::RowsShouldBeNullInRowSet);
        }
        if let Some(aggregates) = &row_set.aggregates {
            match aggregates.get("count").and_then(serde_json::Value::as_u64) {
                None => Err(Error::MissingField("count".into())),
                Some(count) => Ok(count),
            }
        } else {
            Err(Error::AggregatesShouldBeNonNullInRowSet)
        }
    } else {
        Err(Error::ExpectedSingleRowSet)
    }
}

async fn test_column_count_aggregate<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    collection_info: &models::CollectionInfo,
    collection_type: &models::ObjectType,
    total_count: u64,
) -> Result<(), Error> {
    let mut aggregates = IndexMap::new();

    for field_name in collection_type.fields.keys() {
        let aggregate = models::Aggregate::ColumnCount {
            column: field_name.clone(),
            distinct: false,
        };
        aggregates.insert(format!("{}_count", field_name), aggregate);

        let aggregate = models::Aggregate::ColumnCount {
            column: field_name.clone(),
            distinct: true,
        };
        aggregates.insert(format!("{}_distinct_count", field_name), aggregate);
    }

    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: Some(aggregates),
            fields: None,
            limit: Some(10),
            offset: None,
            order_by: None,
            predicate: None,
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };
    let response = execute_and_snapshot_query(configuration, connector, query_request).await?;
    if let [row_set] = &*response.0 {
        if row_set.rows.is_some() {
            return Err(Error::RowsShouldBeNullInRowSet);
        }
        if let Some(aggregates) = &row_set.aggregates {
            for field_name in collection_type.fields.keys() {
                let count_field = format!("{}_count", field_name);
                let count = aggregates
                    .get(count_field.as_str())
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(Error::MissingField(count_field))?;

                let distinct_field = format!("{}_distinct_count", field_name);
                let distinct_count = aggregates
                    .get(distinct_field.as_str())
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(Error::MissingField(distinct_field))?;

                if count > total_count {
                    return Err(Error::ResponseDoesNotSatisfy(format!(
                        "star_count >= column_count({})",
                        field_name
                    )));
                }

                if distinct_count > count {
                    return Err(Error::ResponseDoesNotSatisfy(format!(
                        "column_count >= column_count(distinct {})",
                        field_name
                    )));
                }
            }
        } else {
            return Err(Error::AggregatesShouldBeNonNullInRowSet);
        }
    } else {
        return Err(Error::ExpectedSingleRowSet);
    }
    Ok(())
}
