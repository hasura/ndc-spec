use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::fs::File;
use std::future::Future;
use std::hash::Hasher;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use indexmap::IndexMap;
use ndc_client::apis::configuration::Configuration;
use ndc_client::apis::default_api as api;
use ndc_client::models::{self, OrderDirection};
use proptest::prelude::Rng;
use proptest::sample::select;
use proptest::strategy::{Just, Strategy, Union, ValueTree};
use proptest::test_runner::{Config, Reason, RngAlgorithm, TestRng, TestRunner};
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("error communicating with the connector: {0}")]
    CommunicationError(#[from] ndc_client::apis::Error),
    #[error("error generating test data: {0}")]
    StrategyError(Reason),
    #[error("error parsing semver range: {0}")]
    SemverError(#[from] semver::Error),
    #[error(
        "capabilities.versions does not include the current version of the specification: {0}"
    )]
    IncompatibleSpecification(semver::VersionReq),
    #[error("collection {0} is not a defined collection")]
    CollectionIsNotDefined(String),
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
    #[error("error response from connector: {0:?}")]
    ConnectorError(ndc_client::models::ErrorResponse),
    #[error("cannot open snapshot file: {0:?}")]
    CannotOpenSnapshotFile(std::io::Error),
    #[error("error (de)serializing data structure: {0:?}")]
    SerdeError(serde_json::Error),
    #[error("snapshot did not match file {0}: {1}")]
    ResponseDidNotMatchSnapshot(PathBuf, String),
    #[error("response from connector does not satisfy requirement: {0}")]
    ResponseDoesNotSatisfy(String),
    #[error("other error")]
    OtherError(#[from] Box<dyn std::error::Error>),
}

impl From<Reason> for Error {
    fn from(value: Reason) -> Self {
        Error::StrategyError(value)
    }
}

#[derive(Debug)]
pub struct TestResults {
    pub path: Vec<String>,
    pub failures: Vec<FailedTest>,
}

#[derive(Debug)]
pub struct FailedTest {
    pub path: Vec<String>,
    pub name: String,
    pub error: Error,
}

async fn test<A, F: Future<Output = Result<A, Error>>>(
    name: &str,
    results: &RefCell<TestResults>,
    f: F,
) -> Option<A>
where
    A: serde::Serialize + serde::de::DeserializeOwned + PartialEq,
{
    use colored::Colorize;

    {
        let mut results_mut = results.borrow_mut();
        let level = results_mut.path.len();
        let spaces = "│ ".repeat(level);
        print!("{spaces}├ {name} ...");
        results_mut.path.push(name.into());
    }

    let result = match f.await {
        Ok(result) => {
            println!(" {}", "OK".green());
            Ok(result)
        }
        Err(err) => {
            println!(" {}", "FAIL".red());
            Err(err)
        }
    };

    let mut results_mut = results.borrow_mut();
    results_mut.path.pop();

    match result {
        Err(err) => {
            let path = results_mut.path.clone();
            results_mut.failures.push(FailedTest {
                path,
                name: name.into(),
                error: err,
            });
            None
        }
        Ok(result) => Some(result),
    }
}

async fn nest<A, F: Future<Output = A>>(name: &str, results: &RefCell<TestResults>, f: F) -> A {
    {
        let mut results_mut = results.borrow_mut();
        let level = results_mut.path.len();
        let spaces = "│ ".repeat(level);
        println!("{spaces}├ {name} ...");
        results_mut.path.push(name.into());
    }
    let result = f.await;
    {
        let mut results_mut = results.borrow_mut();
        let _ = results_mut.path.pop();
    }
    result
}

fn snapshot_test<R>(snapshot_path: &Path, expected: &R) -> Result<(), Error>
where
    R: serde::Serialize + serde::de::DeserializeOwned + PartialEq,
{
    if snapshot_path.exists() {
        let snapshot_file = File::open(snapshot_path).map_err(Error::CannotOpenSnapshotFile)?;
        let snapshot: R = serde_json::from_reader(snapshot_file).map_err(Error::SerdeError)?;

        if snapshot != *expected {
            let expected_json =
                serde_json::to_string_pretty(&expected).map_err(Error::SerdeError)?;
            return Err(Error::ResponseDidNotMatchSnapshot(
                snapshot_path.into(),
                expected_json,
            ));
        }
    } else {
        let parent = snapshot_path.parent().unwrap();
        let snapshot_file = (|| {
            std::fs::create_dir_all(parent)?;
            File::create(snapshot_path)
        })()
        .map_err(Error::CannotOpenSnapshotFile)?;

        serde_json::to_writer_pretty(snapshot_file, &expected).map_err(Error::SerdeError)?;
    }

    Ok(())
}

#[derive(Debug)]
pub struct TestConfiguration {
    pub seed: Option<String>,
    pub snapshots_dir: Option<PathBuf>,
}

#[async_trait]
pub trait Connector {
    async fn get_capabilities(&self) -> Result<models::CapabilitiesResponse, Error>;

    async fn get_schema(&self) -> Result<models::SchemaResponse, Error>;

    async fn query(&self, request: models::QueryRequest) -> Result<models::QueryResponse, Error>;

    async fn mutation(
        &self,
        request: models::MutationRequest,
    ) -> Result<models::MutationResponse, Error>;
}

#[async_trait]
impl Connector for Configuration {
    async fn get_capabilities(&self) -> Result<models::CapabilitiesResponse, Error> {
        Ok(api::capabilities_get(self).await?)
    }

    async fn get_schema(&self) -> Result<models::SchemaResponse, Error> {
        Ok(api::schema_get(self).await?)
    }

    async fn query(&self, request: models::QueryRequest) -> Result<models::QueryResponse, Error> {
        Ok(api::query_post(self, request).await?)
    }

    async fn mutation(
        &self,
        request: models::MutationRequest,
    ) -> Result<models::MutationResponse, Error> {
        Ok(api::mutation_post(self, request).await?)
    }
}

pub fn report(results: TestResults) -> String {
    use colored::Colorize;

    let mut result = format!("Failed with {0} test failures:", results.failures.len())
        .red()
        .to_string();

    let mut ix = 1;
    for failure in results.failures {
        result += format!("\n\n[{0}] {1}", ix, failure.name).as_str();
        for path_element in failure.path {
            result += format!("\n  in {0}", path_element).as_str();
        }
        result += format!("\nDetails: {0}", failure.error).as_str();
        ix += 1;
    }

    result
}

pub async fn test_connector<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
) -> TestResults {
    let results = RefCell::new(TestResults {
        path: vec![],
        failures: vec![],
    });

    let mut runner = TestRunner::new_with_rng(
        Config::default(),
        match &configuration.seed {
            Some(seed) => TestRng::from_seed(RngAlgorithm::XorShift, seed.as_bytes()),
            None => TestRng::deterministic_rng(RngAlgorithm::XorShift),
        },
    );

    let _ = run_all_tests(configuration, connector, &mut runner, &results).await;

    results.into_inner()
}

async fn run_all_tests<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    runner: &mut TestRunner,
    results: &RefCell<TestResults>,
) -> Option<()> {
    let capabilities = nest("Capabilities", results, async {
        let capabilities = test("Fetching /capabilities", results, async {
            let response = connector.get_capabilities().await?;
            for snapshots_dir in configuration.snapshots_dir.iter() {
                snapshot_test(snapshots_dir.join("capabilities").as_path(), &response)?;
            }
            Ok(response)
        })
        .await?;

        let _ = test("Validating capabilities", results, async {
            validate_capabilities(&capabilities)
        })
        .await;

        Some(capabilities)
    })
    .await?;

    let schema = nest("Schema", results, async {
        let schema = test("Fetching schema", results, async {
            let response = connector.get_schema().await?;
            for snapshots_dir in configuration.snapshots_dir.iter() {
                snapshot_test(snapshots_dir.join("schema").as_path(), &response)?;
            }
            Ok(response)
        })
        .await?;

        nest("Validating schema", results, async {
            validate_schema(&schema, results).await
        })
        .await?;

        Some(schema)
    })
    .await?;

    nest(
        "Query",
        results,
        test_query(
            configuration,
            connector,
            &capabilities,
            &schema,
            runner,
            results,
        ),
    )
    .await;

    Some(())
}

pub fn validate_capabilities(capabilities: &models::CapabilitiesResponse) -> Result<(), Error> {
    let spec_version = semver::Version::parse(env!("CARGO_PKG_VERSION"))?;
    let claimed_range = semver::VersionReq::parse(capabilities.versions.as_str())?;
    if !claimed_range.matches(&spec_version) {
        return Err(Error::IncompatibleSpecification(claimed_range));
    }

    Ok(())
}

pub async fn validate_schema(
    schema: &models::SchemaResponse,
    results: &RefCell<TestResults>,
) -> Option<()> {
    let _ = test("object_types", results, async {
        for (_type_name, object_type) in schema.object_types.iter() {
            for (_field_name, object_field) in object_type.fields.iter() {
                validate_type(schema, &object_field.r#type)?;
            }
        }
        Ok(())
    })
    .await;

    nest("Collections", results, async {
        for collection_info in schema.collections.iter() {
            nest(collection_info.name.as_str(), results, async {
                let _ = test("Arguments", results, async {
                    for (_arg_name, arg_info) in collection_info.arguments.iter() {
                        validate_type(schema, &arg_info.argument_type)?;
                    }
                    Ok(())
                })
                .await;

                let _ = test("Collection type", results, async {
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

    nest("Functions", results, async {
        for function_info in schema.functions.iter() {
            nest(function_info.name.as_str(), results, async {
                let _ = test("Result type", results, async {
                    validate_type(schema, &function_info.result_type)
                })
                .await;

                let _ = test("Arguments", results, async {
                    for (_arg_name, arg_info) in function_info.arguments.iter() {
                        validate_type(schema, &arg_info.argument_type)?;
                    }

                    Ok(())
                })
                .await;
            })
            .await;
        }

        nest("Procedures", results, async {
            for procedure_info in schema.procedures.iter() {
                nest(procedure_info.name.as_str(), results, async {
                    let _ = test("Result type", results, async {
                        validate_type(schema, &procedure_info.result_type)
                    })
                    .await;

                    let _ = test("Arguments", results, async {
                        for (_arg_name, arg_info) in procedure_info.arguments.iter() {
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
    }

    Ok(())
}

pub async fn test_query<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    capabilities: &models::CapabilitiesResponse,
    schema: &models::SchemaResponse,
    runner: &mut TestRunner,
    results: &RefCell<TestResults>,
) {
    for collection_info in schema.collections.iter() {
        nest(collection_info.name.as_str(), results, async {
            if collection_info.arguments.is_empty() {
                nest("Simple queries", results, async {
                    test_simple_queries(
                        configuration,
                        connector,
                        runner,
                        results,
                        schema,
                        collection_info,
                    )
                    .await
                })
                .await;

                if capabilities.capabilities.relationships.is_some() {
                    nest("Relationship queries", results, async {
                        test_relationship_queries(
                            configuration,
                            connector,
                            results,
                            schema,
                            collection_info,
                        )
                        .await
                    })
                    .await;
                }

                nest("Aggregate queries", results, async {
                    test_aggregate_queries(
                        configuration,
                        connector,
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

async fn test_simple_queries<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    runner: &mut TestRunner,
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

    let rows = test(
        "Select top N",
        results,
        test_select_top_n_rows(configuration, connector, collection_type, collection_info),
    )
    .await?;

    test("Predicates", results, async {
        let value_strategies = make_value_strategies(rows, collection_type)?;

        if let Some(expression_strategy) = make_expression_strategies(value_strategies) {
            for _ in 0..10 {
                if let Ok(tree) = expression_strategy.new_tree(runner) {
                    let predicate = tree.current();

                    test_select_top_n_rows_with_predicate(
                        configuration,
                        connector,
                        predicate,
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

    test("Sorting", results, async {
        if let Some(order_by_elements_strategy) =
            make_order_by_elements_strategy(collection_type.clone())
        {
            for _ in 0..10 {
                if let Ok(tree) = order_by_elements_strategy.new_tree(runner) {
                    let elements = tree.current();
                    test_select_top_n_rows_with_sort(
                        configuration,
                        connector,
                        elements,
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
    predicate: models::Expression,
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
            predicate: Some(predicate),
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };

    let response = execute_and_snapshot_query(configuration, connector, query_request).await?;

    expect_single_non_empty_rows(&response)?;

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

async fn test_relationship_queries<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
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
        nest(foreign_key_name, results, async {
            let _ = test(
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

            let _ = test(
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

fn make_value_strategies(
    rows: Vec<IndexMap<String, models::RowFieldValue>>,
    collection_type: &models::ObjectType,
) -> Result<BTreeMap<String, impl Strategy<Value = serde_json::Value>>, Error> {
    let mut values: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();

    for row in rows {
        for (field_name, _) in collection_type.fields.iter() {
            if !row.contains_key(field_name.as_str()) {
                return Err(Error::MissingField(field_name.clone()));
            }
        }

        for (field_name, field_value) in row {
            if !field_value.0.is_null() {
                values
                    .entry(field_name.clone())
                    .or_insert(vec![])
                    .push(field_value.0.clone());
            }
        }
    }

    let strategies = values
        .into_iter()
        .map(|(field_name, examples)| Ok((field_name, select(examples))))
        .collect::<Result<BTreeMap<String, _>, Reason>>()?;

    Ok(strategies)
}

fn make_expression_strategies<S: Strategy<Value = serde_json::Value>>(
    value_strategies: BTreeMap<String, S>,
) -> Option<impl Strategy<Value = models::Expression>> {
    let expression_strategies = value_strategies
        .into_iter()
        .map(|(field_name, strategy)| {
            strategy.prop_map(move |value| models::Expression::BinaryComparisonOperator {
                column: models::ComparisonTarget::Column {
                    name: field_name.clone(),
                    path: vec![],
                },
                operator: models::BinaryComparisonOperator::Equal,
                value: models::ComparisonValue::Scalar { value },
            })
        })
        .collect::<Vec<_>>();

    if expression_strategies.is_empty() {
        None
    } else {
        Some(Union::new(expression_strategies))
    }
}

fn make_order_by_elements_strategy(
    collection_type: models::ObjectType,
) -> Option<impl Strategy<Value = Vec<models::OrderByElement>>> {
    if collection_type.fields.is_empty() {
        None
    } else {
        let random_fields =
            Just(collection_type.fields.keys().cloned().collect::<Vec<_>>()).prop_shuffle();
        let strategy = random_fields.prop_perturb(|fields, mut rng| {
            let mut elements = vec![];

            let fields = fields.into_iter().take(rng.gen_range(0..3));

            let order_direction = if rng.gen_bool(0.5) {
                OrderDirection::Asc
            } else {
                OrderDirection::Desc
            };

            for field in fields {
                elements.push(models::OrderByElement {
                    order_direction,
                    target: models::OrderByTarget::Column {
                        name: field.clone(),
                        path: vec![],
                    },
                });
            }

            elements
        });

        Some(strategy)
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
                    fields: None
                },
            )
        })
        .collect::<IndexMap<String, models::Field>>()
}

async fn test_aggregate_queries<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
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

    let total_count = test("star_count", results, async {
        test_star_count_aggregate(configuration, connector, collection_info).await
    })
    .await?;

    let _ = test("column_count", results, async {
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
                None => {
                    Err(Error::MissingField("count".into()))
                }
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
                    return Err(Error::ResponseDoesNotSatisfy(format!("star_count >= column_count({})", field_name)));
                }
                
                if distinct_count > count {
                    return Err(Error::ResponseDoesNotSatisfy(format!("column_count >= column_count(distinct {})", field_name)));
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

pub async fn test_snapshots_in_directory<C: Connector>(
    connector: &C,
    snapshots_dir: PathBuf,
) -> TestResults {
    let results = RefCell::new(TestResults {
        path: vec![],
        failures: vec![],
    });

    let _ = async {
        nest(
            "Query",
            &results,
            test_snapshots_in_directory_with::<C, _, _, _>(
                snapshots_dir.join("query"),
                &results,
                |req| connector.query(req),
            ),
        )
        .await;

        nest(
            "Mutation",
            &results,
            test_snapshots_in_directory_with::<C, _, _, _>(
                snapshots_dir.join("mutation"),
                &results,
                |req| connector.mutation(req),
            ),
        )
        .await;

        Some(())
    }
    .await;

    results.into_inner()
}

pub async fn test_snapshots_in_directory_with<
    C: Connector,
    Req: DeserializeOwned,
    Res: DeserializeOwned + serde::Serialize + PartialEq,
    F: Future<Output = Result<Res, Error>>,
>(
    snapshots_dir: PathBuf,
    results: &RefCell<TestResults>,
    f: impl Fn(Req) -> F,
) {
    match std::fs::read_dir(snapshots_dir) {
        Ok(dir) => {
            for entry in dir {
                let entry = entry.expect("Error reading snapshot directory entry");

                test(
                    entry.file_name().to_str().unwrap_or("{unknown}"),
                    results,
                    async {
                        let path = entry.path();

                        let snapshot_pathbuf = path.to_path_buf().join("expected.json");
                        let snapshot_path = snapshot_pathbuf.as_path();

                        let request_file = File::open(path.join("request.json"))
                            .map_err(Error::CannotOpenSnapshotFile)?;
                        let request =
                            serde_json::from_reader(request_file).map_err(Error::SerdeError)?;

                        let response = f(request)
                            .await?;

                        snapshot_test(snapshot_path, &response)
                    },
                )
                .await;
            }
        }
        Err(e) => println!("Warning: a snapshot folder could not be found: {}", e),
    }
}
