use std::collections::{BTreeMap, HashMap};
use std::future::Future;

use indexmap::IndexMap;
use ndc_client::apis::configuration::Configuration;
use ndc_client::apis::default_api as api;
use ndc_client::models;
use proptest::sample::select;
use proptest::strategy::{Strategy, Union, ValueTree};
use proptest::test_runner::{Config, Reason, RngAlgorithm, TestRng, TestRunner};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TestFailure {
    #[error("error communicating with the connector: {0}")]
    CommunicationError(ndc_client::apis::Error),
    #[error("error generating test data: {0}")]
    StrategyError(Reason),
    #[error("collection type {0} is not a defined object type")]
    CollectionTypeIsNotDefined(String),
    #[error("named type {0} is not a defined object or scalar type")]
    NamedTypeIsNotDefined(String),
    #[error("insertable column {0} is not defined on collection type")]
    InsertableColumnNotDefined(String),
    #[error("updatable column {0} is not defined on collection type")]
    UpdatableColumnNotDefined(String),
    #[error("expected null rows in RowSet")]
    RowsShouldBeNullInRowSet,
    #[error("expected non-null rows in RowSet")]
    RowsShouldBeNonNullInRowSet,
    #[error("expected null aggregates in RowSet")]
    AggregatesShouldBeNullInRowSet,
    #[error("expected non-null aggregates in RowSet")]
    AggregatesShouldBeNonNullInRowSet,
    #[error("expected a single RowSet in the QueryResponse")]
    ExpectedSingleRowSet,
    #[error("expected non-empty rows in RowSet")]
    ExpectedNonEmptyRows,
    #[error("requested field {0} was missing in response")]
    MissingField(String),
}

impl From<ndc_client::apis::Error> for TestFailure {
    fn from(value: ndc_client::apis::Error) -> Self {
        TestFailure::CommunicationError(value)
    }
}

impl From<Reason> for TestFailure {
    fn from(value: Reason) -> Self {
        TestFailure::StrategyError(value)
    }
}

async fn test<A, F: Future<Output = Result<A, TestFailure>>>(
    name: &str,
    level: usize,
    f: F,
) -> Result<A, TestFailure> {
    let spaces = "  ".repeat(level);
    print!("{spaces}∟ {name} ...");

    match f.await {
        Ok(result) => {
            println!(" \x1b[1;32mOK\x1b[22;0m");
            Ok(result)
        }
        Err(err) => {
            println!(" \x1b[1;31mFAIL\x1b[22;0m");
            eprintln!("{name} failed with message {}", err.to_string());
            Err(err)
        }
    }
}

fn nest(name: &str, level: usize) {
    let spaces = "  ".repeat(level);
    println!("{spaces}∟ {name} ...");
}

pub async fn test_connector(configuration: &Configuration) -> Result<(), TestFailure> {
    let mut runner = TestRunner::new_with_rng(
        Config::default(),
        match &configuration.seed {
            Some(seed) => TestRng::from_seed(RngAlgorithm::XorShift, &seed.as_bytes()),
            None => TestRng::deterministic_rng(RngAlgorithm::XorShift),
        },
    );

    println!("Capabilities");
    let capabilities = async {
        let capabilities = test("Fetching /capabilities ...", 1, async {
            Ok(api::capabilities_get(configuration).await?)
        })
        .await?;

        let _ = test("Validating capabilities", 1, async {
            validate_capabilities(&capabilities)
        })
        .await;

        Ok::<_, TestFailure>(capabilities)
    }
    .await?;

    println!("Schema");
    let schema = async {
        let schema = test("Fetching /schema", 1, async {
            Ok(api::schema_get(configuration).await?)
        })
        .await?;

        nest("Validating schema", 1);
        let _ = validate_schema(&schema).await;

        Ok::<_, TestFailure>(schema)
    }
    .await?;

    println!("Query");

    test_query(configuration, &capabilities, &schema, &mut runner).await;

    Ok(())
}

pub fn validate_capabilities(
    _capabilities: &models::CapabilitiesResponse,
) -> Result<(), TestFailure> {
    // TODO: validate capabilities.version
    Ok(())
}

pub async fn validate_schema(schema: &models::SchemaResponse) -> Result<(), TestFailure> {
    let _ = test("object_types", 2, async {
        for (_type_name, object_type) in schema.object_types.iter() {
            for (_field_name, object_field) in object_type.fields.iter() {
                validate_type(schema, &object_field.r#type)?;
            }
        }
        Ok(())
    })
    .await;

    nest("Collections", 2);

    for collection_info in schema.collections.iter() {
        nest(collection_info.name.as_str(), 3);

        let _ = test("Arguments", 4, async {
            for (_arg_name, arg_info) in collection_info.arguments.iter() {
                validate_type(schema, &arg_info.argument_type)?;
            }
            Ok(())
        })
        .await;

        let _ = test("Collection type", 4, async {
            let collection_type = schema
                .object_types
                .get(collection_info.collection_type.as_str())
                .ok_or(TestFailure::CollectionTypeIsNotDefined(
                    collection_info.collection_type.clone(),
                ))?;

            if let Some(insertable_columns) = &collection_info.insertable_columns {
                for insertable_column in insertable_columns.iter() {
                    if !collection_type
                        .fields
                        .contains_key(insertable_column.as_str())
                    {
                        return Err(TestFailure::InsertableColumnNotDefined(
                            insertable_column.clone(),
                        ));
                    }
                }
            }
            if let Some(updatable_columns) = &collection_info.updatable_columns {
                for updatable_column in updatable_columns.iter() {
                    if !collection_type
                        .fields
                        .contains_key(updatable_column.as_str())
                    {
                        return Err(TestFailure::UpdatableColumnNotDefined(
                            updatable_column.clone(),
                        ));
                    }
                }
            }

            Ok(())
        })
        .await;
    }

    nest("Functions", 2);

    for function_info in schema.functions.iter() {
        nest(function_info.name.as_str(), 3);

        let _ = test("Result type", 4, async {
            validate_type(schema, &function_info.result_type)
        })
        .await;

        let _ = test("Arguments", 4, async {
            for (_arg_name, arg_info) in function_info.arguments.iter() {
                validate_type(schema, &arg_info.argument_type)?;
            }

            Ok(())
        })
        .await;
    }

    nest("Procedures", 2);

    for procedure_info in schema.procedures.iter() {
        nest(procedure_info.name.as_str(), 3);

        let _ = test("Result type", 4, async {
            validate_type(schema, &procedure_info.result_type)
        })
        .await;

        let _ = test("Arguments", 4, async {
            for (_arg_name, arg_info) in procedure_info.arguments.iter() {
                validate_type(schema, &arg_info.argument_type)?;
            }

            Ok(())
        })
        .await;
    }

    Ok(())
}

pub fn validate_type(
    schema: &models::SchemaResponse,
    r#type: &models::Type,
) -> Result<(), TestFailure> {
    match r#type {
        models::Type::Named { name } => {
            if !schema.object_types.contains_key(name.as_str())
                && !schema.scalar_types.contains_key(name.as_str())
            {
                return Err(TestFailure::NamedTypeIsNotDefined(name.clone()));
            }
        }
        models::Type::Array { element_type } => {
            validate_type(schema, element_type)?;
        }
        models::Type::Nullable { underlying_type } => {
            validate_type(schema, underlying_type)?;
        }
    }

    Ok(())
}

pub async fn test_query(
    configuration: &Configuration,
    _capabilities: &models::CapabilitiesResponse,
    schema: &models::SchemaResponse,
    runner: &mut TestRunner,
) {
    for collection_info in schema.collections.iter() {
        nest(collection_info.name.as_str(), 1);

        if collection_info.arguments.is_empty() {
            nest("Simple queries", 2);

            let _ = test_simple_queries(runner, configuration, schema, collection_info).await;

            nest("Aggregate queries", 2);

            let _ = test_aggregate_queries(configuration, schema, collection_info).await;
        } else {
            eprintln!("Skipping parameterized collection {}", collection_info.name);
        }
    }
}

async fn test_simple_queries(
    runner: &mut TestRunner,
    configuration: &Configuration,
    schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
) -> Result<(), TestFailure> {
    let collection_type = schema
        .object_types
        .get(collection_info.collection_type.as_str())
        .ok_or(TestFailure::CollectionTypeIsNotDefined(
            collection_info.collection_type.clone(),
        ))?;

    let rows = test("Select top N", 3, test_select_top_n_rows(collection_type, collection_info, configuration)).await?;

    let value_strategies = make_value_strategies(rows, collection_type)?;

    if let Some(expression_strategy) = make_expression_strategies(value_strategies) {
        let _ = test("Predicates", 3, async {
            for _ in 1..10 {
                test_select_top_n_rows_with_predicate(
                    runner,
                    &expression_strategy,
                    collection_type,
                    collection_info,
                    configuration,
                )
                .await?;
            }

            Ok(())
        }).await?;
    } else {
        eprintln!("Skipping empty collection {}", collection_info.name);
    }

    Ok(())
}

async fn test_select_top_n_rows(
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
    configuration: &Configuration,
) -> Result<Vec<IndexMap<String, models::RowFieldValue>>, TestFailure> {
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

    let response = api::query_post(configuration, query_request)
        .await?;

    if response.0.len() != 1 {
        return Err(TestFailure::ExpectedSingleRowSet);
    }

    let row_set = response.0[0].clone();

    row_set.rows.ok_or(TestFailure::RowsShouldBeNonNullInRowSet)
}

async fn test_select_top_n_rows_with_predicate(
    runner: &mut TestRunner,
    expression_strategy: &impl Strategy<Value = models::Expression>,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
    configuration: &Configuration,
) -> Result<(), TestFailure> {
    if let Ok(tree) = expression_strategy.new_tree(runner) {
        let predicate = tree.current();

        let fields = select_all_columns(collection_type);

        let query_request = models::QueryRequest {
            collection: collection_info.name.clone(),
            query: models::Query {
                aggregates: None,
                fields: Some(fields),
                limit: Some(10),
                offset: None,
                order_by: None,
                predicate: Some(predicate),
            },
            arguments: BTreeMap::new(),
            collection_relationships: BTreeMap::new(),
            variables: None,
        };

        let response = api::query_post(configuration, query_request)
            .await?;

        if response.0.len() != 1 {
            return Err(TestFailure::ExpectedSingleRowSet);
        }

        let row_set = response.0.first().unwrap();
        let rows = row_set
            .rows
            .as_ref()
            .ok_or(TestFailure::RowsShouldBeNonNullInRowSet)?;

        if rows.is_empty() {
            return Err(TestFailure::ExpectedNonEmptyRows);
        }
    }

    Ok(())
}

fn make_value_strategies(
    rows: Vec<IndexMap<String, models::RowFieldValue>>,
    collection_type: &models::ObjectType,
) -> Result<HashMap<String, impl Strategy<Value = serde_json::Value>>, TestFailure> {
    let mut values: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

    for row in rows {
        for (field_name, _) in collection_type.fields.iter() {
            if !row.contains_key(field_name.as_str()) {
                panic!("field {0} was missing in query response", field_name)
            }
        }

        for (field_name, field_value) in row {
            values
                .entry(field_name.clone())
                .or_insert(vec![])
                .push(field_value.0.clone());
        }
    }

    let strategies = values
        .into_iter()
        .map(|(field_name, examples)| Ok((field_name, select(examples))))
        .collect::<Result<HashMap<String, _>, Reason>>()?;

    Ok(strategies)
}

fn make_expression_strategies<S: Strategy<Value = serde_json::Value>>(
    value_strategies: HashMap<String, S>,
) -> Option<impl Strategy<Value = models::Expression>> {
    let expression_strategies = value_strategies
        .into_iter()
        .map(|(field_name, strategy)| {
            strategy.prop_map(move |value| models::Expression::BinaryComparisonOperator {
                column: Box::new(models::ComparisonTarget::Column {
                    name: field_name.clone(),
                    path: vec![],
                }),
                operator: Box::new(models::BinaryComparisonOperator::Equal),
                value: Box::new(models::ComparisonValue::Scalar { value }),
            })
        })
        .collect::<Vec<_>>();

    if expression_strategies.is_empty() {
        None
    } else {
        Some(Union::new(expression_strategies))
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
                },
            )
        })
        .collect::<IndexMap<String, models::Field>>()
}

async fn test_aggregate_queries(
    configuration: &Configuration,
    _schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
) -> Result<(), TestFailure> {
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
    let response = api::query_post(configuration, query_request).await.unwrap();
    
    if let [row_set] = &*response.0 {
        if row_set.rows.is_some() {
            return Err(TestFailure::RowsShouldBeNullInRowSet);
        }
        if let Some(aggregates) = &row_set.aggregates {
            if !aggregates.contains_key("count") {
                return Err(TestFailure::MissingField("count".into()));
            }
        } else {
            return Err(TestFailure::AggregatesShouldBeNonNullInRowSet);
        }
    } else {
        return Err(TestFailure::ExpectedSingleRowSet);
    }

    Ok(())
}
