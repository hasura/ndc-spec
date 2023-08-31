use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::future::Future;

use indexmap::IndexMap;
use ndc_client::apis::configuration::Configuration;
use ndc_client::apis::default_api as api;
use ndc_client::models::{self, OrderDirection};
use proptest::prelude::Rng;
use proptest::sample::select;
use proptest::strategy::{Just, Strategy, Union, ValueTree};
use proptest::test_runner::{Config, Reason, RngAlgorithm, TestRng, TestRunner};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("error communicating with the connector: {0}")]
    CommunicationError(ndc_client::apis::Error),
    #[error("error generating test data: {0}")]
    StrategyError(Reason),
    #[error("error parsing semver range: {0}")]
    SemverError(semver::Error),
    #[error(
        "capabilities.versions does not include the current version of the specification: {0}"
    )]
    IncompatibleSpecification(semver::VersionReq),
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

impl From<ndc_client::apis::Error> for Error {
    fn from(value: ndc_client::apis::Error) -> Self {
        Error::CommunicationError(value)
    }
}

impl From<Reason> for Error {
    fn from(value: Reason) -> Self {
        Error::StrategyError(value)
    }
}

impl From<semver::Error> for Error {
    fn from(value: semver::Error) -> Self {
        Error::SemverError(value)
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
) -> Option<A> {
    {
        let results = results.borrow();
        let level = results.path.len();
        let spaces = "│ ".repeat(level);
        print!("{spaces}├ {name} ...");
    }

    match f.await {
        Ok(result) => {
            println!(" \x1b[1;32mOK\x1b[22;0m");
            Some(result)
        }
        Err(err) => {
            let mut results_mut = results.borrow_mut();
            println!(" \x1b[1;31mFAIL\x1b[22;0m");
            let path = results_mut.path.clone();
            results_mut.failures.push(FailedTest {
                path,
                name: name.into(),
                error: err,
            });
            None
        }
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

pub async fn test_connector(configuration: &Configuration) -> TestResults {
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

    let _ = run_all_tests(&configuration, &mut runner, &results).await;

    results.into_inner()
}

async fn run_all_tests(
    configuration: &&Configuration,
    runner: &mut TestRunner,
    results: &RefCell<TestResults>,
) -> Option<()> {
    println!("Capabilities");

    let capabilities = async {
        let capabilities = test("Fetching /capabilities ...", results, async {
            Ok(api::capabilities_get(configuration).await?)
        })
        .await?;

        let _ = test("Validating capabilities", results, async {
            validate_capabilities(&capabilities)
        })
        .await;

        Some(capabilities)
    }
    .await?;

    println!("Schema");
    let schema = async {
        let schema = test("Fetching /schema", results, async {
            Ok(api::schema_get(configuration).await?)
        })
        .await?;

        nest("Validating schema", results, async {
            validate_schema(&schema, results).await
        })
        .await?;

        Some(schema)
    }
    .await?;

    println!("Query");

    test_query(configuration, &capabilities, &schema, runner, results).await;

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
                    let collection_type = schema
                        .object_types
                        .get(collection_info.collection_type.as_str())
                        .ok_or(Error::CollectionTypeIsNotDefined(
                            collection_info.collection_type.clone(),
                        ))?;

                    if let Some(insertable_columns) = &collection_info.insertable_columns {
                        for insertable_column in insertable_columns.iter() {
                            if !collection_type
                                .fields
                                .contains_key(insertable_column.as_str())
                            {
                                return Err(Error::InsertableColumnNotDefined(
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
                                return Err(Error::UpdatableColumnNotDefined(
                                    updatable_column.clone(),
                                ));
                            }
                        }
                    }

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

pub async fn test_query(
    configuration: &Configuration,
    _capabilities: &models::CapabilitiesResponse,
    schema: &models::SchemaResponse,
    runner: &mut TestRunner,
    results: &RefCell<TestResults>,
) {
    for collection_info in schema.collections.iter() {
        nest(collection_info.name.as_str(), results, async {
            if collection_info.arguments.is_empty() {
                nest("Simple queries", results, async {
                    test_simple_queries(runner, results, configuration, schema, collection_info)
                        .await
                })
                .await;

                nest("Aggregate queries", results, async {
                    test_aggregate_queries(configuration, schema, collection_info, results).await
                })
                .await;
            } else {
                eprintln!("Skipping parameterized collection {}", collection_info.name);
            }
        })
        .await;
    }
}

async fn test_simple_queries(
    runner: &mut TestRunner,
    results: &RefCell<TestResults>,
    configuration: &Configuration,
    schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
) -> Option<()> {
    let (collection_type, rows) = test("Select top N", results, async {
        let collection_type = schema
            .object_types
            .get(collection_info.collection_type.as_str())
            .ok_or(Error::CollectionTypeIsNotDefined(
                collection_info.collection_type.clone(),
            ))?;

        let rows = test_select_top_n_rows(collection_type, collection_info, configuration).await?;

        Ok((collection_type, rows))
    })
    .await?;

    test("Predicates", results, async {
        let value_strategies = make_value_strategies(rows, collection_type)?;

        if let Some(expression_strategy) = make_expression_strategies(value_strategies) {
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
            for _ in 1..10 {
                test_select_top_n_rows_with_sort(
                    runner,
                    &order_by_elements_strategy,
                    collection_type,
                    collection_info,
                    configuration,
                )
                .await?;
            }
        } else {
            eprintln!("Skipping empty collection {}", collection_info.name);
        }

        Ok(())
    })
    .await
}

async fn test_select_top_n_rows(
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
    configuration: &Configuration,
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

    let response = api::query_post(configuration, query_request).await?;

    expect_single_rows(response)
}

async fn test_select_top_n_rows_with_predicate(
    runner: &mut TestRunner,
    expression_strategy: &impl Strategy<Value = models::Expression>,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
    configuration: &Configuration,
) -> Result<(), Error> {
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

        let response = api::query_post(configuration, query_request).await?;

        expect_single_non_empty_rows(response)?;
    }

    Ok(())
}

async fn test_select_top_n_rows_with_sort(
    runner: &mut TestRunner,
    order_by_elements_strategy: &impl Strategy<Value = Vec<models::OrderByElement>>,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
    configuration: &Configuration,
) -> Result<(), Error> {
    if let Ok(tree) = order_by_elements_strategy.new_tree(runner) {
        let elements = tree.current();

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

        let response = api::query_post(configuration, query_request).await?;

        expect_single_non_empty_rows(response)?;
    }

    Ok(())
}

fn expect_single_non_empty_rows(
    response: models::QueryResponse,
) -> Result<Vec<IndexMap<String, models::RowFieldValue>>, Error> {
    let rows = expect_single_rows(response)?;

    if rows.is_empty() {
        return Err(Error::ExpectedNonEmptyRows);
    }

    Ok(rows)
}

fn expect_single_rows(
    response: models::QueryResponse,
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
) -> Result<HashMap<String, impl Strategy<Value = serde_json::Value>>, Error> {
    let mut values: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

    for row in rows {
        for (field_name, _) in collection_type.fields.iter() {
            if !row.contains_key(field_name.as_str()) {
                return Err(Error::MissingField(field_name.clone()));
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

fn make_order_by_elements_strategy(
    collection_type: models::ObjectType,
) -> Option<impl Strategy<Value = Vec<models::OrderByElement>>> {
    if collection_type.fields.is_empty() {
        None
    } else {
        let random_fields = Just(collection_type.fields.keys().cloned().collect::<Vec<_>>()).prop_shuffle();
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
                },
            )
        })
        .collect::<IndexMap<String, models::Field>>()
}

async fn test_aggregate_queries(
    configuration: &Configuration,
    _schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
    results: &RefCell<TestResults>,
) -> Option<()> {
    test("star_count", results, async {
        test_star_count_aggregate(collection_info, configuration).await
    })
    .await
}

async fn test_star_count_aggregate(
    collection_info: &models::CollectionInfo,
    configuration: &Configuration,
) -> Result<(), Error> {
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
            return Err(Error::RowsShouldBeNullInRowSet);
        }
        if let Some(aggregates) = &row_set.aggregates {
            if !aggregates.contains_key("count") {
                return Err(Error::MissingField("count".into()));
            }
        } else {
            return Err(Error::AggregatesShouldBeNonNullInRowSet);
        }
    } else {
        return Err(Error::ExpectedSingleRowSet);
    }
    Ok(())
}
