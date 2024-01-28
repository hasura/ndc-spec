use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashSet},
    error::Error,
    fs::File,
    io::{self, BufRead},
    ops::Deref,
    sync::Arc,
};

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use indexmap::IndexMap;
use ndc_client::models::{self, LeafCapability, RelationshipCapabilities};
use prometheus::{Encoder, IntCounter, IntGauge, Opts, Registry, TextEncoder};
use regex::Regex;
use serde_json::Value;
use tokio::sync::Mutex;

// ANCHOR: row-type
type Row = BTreeMap<String, serde_json::Value>;
// ANCHOR_END: row-type
// ANCHOR: app-state
#[derive(Debug, Clone)]
pub struct AppState {
    pub articles: BTreeMap<i64, Row>,
    pub authors: BTreeMap<i64, Row>,
    pub universities: BTreeMap<i64, Row>,
    pub metrics: Metrics,
}
// ANCHOR_END: app-state

// ANCHOR: read_json_lines
fn read_json_lines(path: &str) -> core::result::Result<BTreeMap<i64, Row>, Box<dyn Error>> {
    let file = File::open(path)?;
    let lines = io::BufReader::new(file).lines();
    let mut records: BTreeMap<i64, Row> = BTreeMap::new();
    for line in lines {
        let row: BTreeMap<String, serde_json::Value> = serde_json::from_str(&line?)?;
        let id = row
            .get("id")
            .ok_or("'id' field not found in json file")?
            .as_i64()
            .ok_or("'id' field was not an integer in json file")?;
        records.insert(id, row);
    }
    Ok(records)
}
// ANCHOR_END: read_json_lines

#[derive(Debug, Clone)]
pub struct Metrics {
    pub registry: Registry,
    pub total_requests: IntCounter,
    pub active_requests: IntGauge,
}

impl Metrics {
    fn new() -> prometheus::Result<Metrics> {
        let total_requests =
            IntCounter::with_opts(Opts::new("total_requests", "number of total requests"))?;
        let active_requests =
            IntGauge::with_opts(Opts::new("active_requests", "number of active requests"))?;
        let registry = Registry::new();
        registry.register(Box::new(total_requests.clone()))?;
        registry.register(Box::new(active_requests.clone()))?;
        Ok(Metrics {
            registry,
            total_requests,
            active_requests,
        })
    }

    fn as_text(&self) -> Option<String> {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).ok()?;
        String::from_utf8(buffer).ok()
    }
}

// ANCHOR: metrics_middleware
async fn metrics_middleware<T>(
    state: State<Arc<Mutex<AppState>>>,
    request: axum::http::Request<T>,
    next: axum::middleware::Next<T>,
) -> axum::response::Response {
    // Don't hold the lock to update metrics, since the
    // lock doesn't protect the metrics anyway.
    let metrics = {
        let state = state.lock().await;
        state.metrics.clone()
    };

    metrics.total_requests.inc();
    metrics.active_requests.inc();
    let response = next.run(request).await;
    metrics.active_requests.dec();
    response
}
// ANCHOR_END: metrics_middleware
// ANCHOR: init_app_state
fn init_app_state() -> AppState {
    // Read the JSON data files
    let articles = read_json_lines("articles.json").unwrap();
    let authors = read_json_lines("authors.json").unwrap();
    let universities = read_json_lines("universities.json").unwrap();

    let metrics = Metrics::new().unwrap();

    AppState {
        articles,
        authors,
        universities,
        metrics,
    }
}
// ANCHOR_END: init_app_state

type Result<A> = core::result::Result<A, (StatusCode, Json<models::ErrorResponse>)>;

// ANCHOR: main
#[tokio::main]
async fn main() {
    let app_state = Arc::new(Mutex::new(init_app_state()));

    let app = Router::new()
        .route("/healthz", get(get_healthz))
        .route("/metrics", get(get_metrics))
        .route("/capabilities", get(get_capabilities))
        .route("/schema", get(get_schema))
        .route("/query", post(post_query))
        .route("/mutation", post(post_mutation))
        .route("/explain", post(post_explain))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            metrics_middleware,
        ))
        .with_state(app_state);

    // run it with hyper on localhost:8100
    axum::Server::bind(&"0.0.0.0:8100".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
// ANCHOR_END: main
// ANCHOR: health
async fn get_healthz() -> StatusCode {
    StatusCode::NO_CONTENT
}
// ANCHOR_END: health
// ANCHOR: metrics
async fn get_metrics(State(state): State<Arc<Mutex<AppState>>>) -> Result<String> {
    let state = state.lock().await;
    state.metrics.as_text().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(models::ErrorResponse {
            message: "cannot encode metrics".into(),
            details: serde_json::Value::Null,
        }),
    ))
}
// ANCHOR_END: metrics
// ANCHOR: capabilities
async fn get_capabilities() -> Json<models::CapabilitiesResponse> {
    Json(models::CapabilitiesResponse {
        versions: "^0.1.0".into(),
        capabilities: models::Capabilities {
            explain: None,
            query: models::QueryCapabilities {
                aggregates: Some(LeafCapability {}),
                variables: Some(LeafCapability {}),
            },
            relationships: Some(RelationshipCapabilities {
                order_by_aggregate: Some(LeafCapability {}),
                relation_comparisons: Some(LeafCapability {}),
            }),
        },
    })
}
// ANCHOR_END: capabilities
// ANCHOR: schema1
async fn get_schema() -> Json<models::SchemaResponse> {
    // ANCHOR_END: schema1
    // ANCHOR: schema_scalar_types
    let scalar_types = BTreeMap::from_iter([
        (
            "String".into(),
            models::ScalarType {
                aggregate_functions: BTreeMap::new(),
                comparison_operators: BTreeMap::from_iter([(
                    "like".into(),
                    models::ComparisonOperatorDefinition {
                        argument_type: models::Type::Named {
                            name: "String".into(),
                        },
                    },
                )]),
            },
        ),
        (
            "Int".into(),
            models::ScalarType {
                aggregate_functions: BTreeMap::from_iter([
                    (
                        "max".into(),
                        models::AggregateFunctionDefinition {
                            result_type: models::Type::Nullable {
                                underlying_type: Box::new(models::Type::Named {
                                    name: "Int".into(),
                                }),
                            },
                        },
                    ),
                    (
                        "min".into(),
                        models::AggregateFunctionDefinition {
                            result_type: models::Type::Nullable {
                                underlying_type: Box::new(models::Type::Named {
                                    name: "Int".into(),
                                }),
                            },
                        },
                    ),
                ]),
                comparison_operators: BTreeMap::from_iter([]),
            },
        ),
    ]);
    // ANCHOR_END: schema_scalar_types
    // ANCHOR: schema_object_type_article
    let article_type = models::ObjectType {
        description: Some("An article".into()),
        fields: BTreeMap::from_iter([
            (
                "id".into(),
                models::ObjectField {
                    description: Some("The article's primary key".into()),
                    r#type: models::Type::Named { name: "Int".into() },
                },
            ),
            (
                "title".into(),
                models::ObjectField {
                    description: Some("The article's title".into()),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "author_id".into(),
                models::ObjectField {
                    description: Some("The article's author ID".into()),
                    r#type: models::Type::Named { name: "Int".into() },
                },
            ),
        ]),
    };
    // ANCHOR_END: schema_object_type_article
    // ANCHOR: schema_object_type_author
    let author_type = models::ObjectType {
        description: Some("An author".into()),
        fields: BTreeMap::from_iter([
            (
                "id".into(),
                models::ObjectField {
                    description: Some("The author's primary key".into()),
                    r#type: models::Type::Named { name: "Int".into() },
                },
            ),
            (
                "first_name".into(),
                models::ObjectField {
                    description: Some("The author's first name".into()),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "last_name".into(),
                models::ObjectField {
                    description: Some("The author's last name".into()),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
        ]),
    };
    // ANCHOR_END: schema_object_type_author
    // ANCHOR: schema_object_types
    let object_types = BTreeMap::from_iter([
        ("article".into(), article_type),
        ("author".into(), author_type),
    ]);
    // ANCHOR_END: schema_object_types
    // ANCHOR: schema_collection_article
    let articles_collection = models::CollectionInfo {
        name: "articles".into(),
        description: Some("A collection of articles".into()),
        collection_type: "article".into(),
        arguments: BTreeMap::new(),
        foreign_keys: BTreeMap::from_iter([(
            "Article_AuthorID".into(),
            models::ForeignKeyConstraint {
                foreign_collection: "authors".into(),
                column_mapping: BTreeMap::from_iter([("author_id".into(), "id".into())]),
            },
        )]),
        uniqueness_constraints: BTreeMap::from_iter([(
            "ArticleByID".into(),
            models::UniquenessConstraint {
                unique_columns: vec!["id".into()],
            },
        )]),
    };
    // ANCHOR_END: schema_collection_article
    // ANCHOR: schema_collection_author
    let authors_collection = models::CollectionInfo {
        name: "authors".into(),
        description: Some("A collection of authors".into()),
        collection_type: "author".into(),
        arguments: BTreeMap::new(),
        foreign_keys: BTreeMap::new(),
        uniqueness_constraints: BTreeMap::from_iter([(
            "AuthorByID".into(),
            models::UniquenessConstraint {
                unique_columns: vec!["id".into()],
            },
        )]),
    };
    // ANCHOR_END: schema_collection_author
    // ANCHOR: schema_collection_articles_by_author
    let articles_by_author_collection = models::CollectionInfo {
        name: "articles_by_author".into(),
        description: Some("Articles parameterized by author".into()),
        collection_type: "article".into(),
        arguments: BTreeMap::from_iter([(
            "author_id".into(),
            models::ArgumentInfo {
                argument_type: models::Type::Named { name: "Int".into() },
                description: None,
            },
        )]),
        foreign_keys: BTreeMap::new(),
        uniqueness_constraints: BTreeMap::new(),
    };
    // ANCHOR_END: schema_collection_articles_by_author
    // ANCHOR: schema_collections
    let collections = vec![
        articles_collection,
        authors_collection,
        articles_by_author_collection,
    ];
    // ANCHOR_END: schema_collections
    // ANCHOR: schema_procedure_upsert_article
    let upsert_article = models::ProcedureInfo {
        name: "upsert_article".into(),
        description: Some("Insert or update an article".into()),
        arguments: BTreeMap::from_iter([(
            "article".into(),
            models::ArgumentInfo {
                description: Some("The article to insert or update".into()),
                argument_type: models::Type::Named {
                    name: "article".into(),
                },
            },
        )]),
        result_type: models::Type::Nullable {
            underlying_type: Box::new(models::Type::Named {
                name: "article".into(),
            }),
        },
    };
    // ANCHOR_END: schema_procedure_upsert_article
    // ANCHOR: schema_procedures
    let procedures = vec![upsert_article];
    // ANCHOR_END: schema_procedures
    // ANCHOR: schema_function_latest_article_id
    let latest_article_id_function = models::FunctionInfo {
        name: "latest_article_id".into(),
        description: Some("Get the ID of the most recent article".into()),
        result_type: models::Type::Nullable {
            underlying_type: Box::new(models::Type::Named { name: "Int".into() }),
        },
        arguments: BTreeMap::new(),
    };
    // ANCHOR_END: schema_function_latest_article_id
    // ANCHOR: schema_functions
    let functions: Vec<models::FunctionInfo> = vec![latest_article_id_function];
    // ANCHOR_END: schema_functions
    // ANCHOR: schema2
    Json(models::SchemaResponse {
        scalar_types,
        object_types,
        collections,
        functions,
        procedures,
    })
}
// ANCHOR_END: schema2
// ANCHOR: post_query
// ANCHOR: post_query_signature
pub async fn post_query(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(request): Json<models::QueryRequest>,
) -> Result<Json<models::QueryResponse>> {
    // ANCHOR_END: post_query_signature
    let state = state.lock().await;

    let variable_sets = request.variables.unwrap_or(vec![BTreeMap::new()]);

    let mut row_sets = vec![];

    for variables in variable_sets.iter() {
        let row_set = execute_query_with_variables(
            &request.collection,
            &request.arguments,
            &request.collection_relationships,
            &request.query,
            variables,
            &state,
        )?;
        row_sets.push(row_set);
    }

    Ok(Json(models::QueryResponse(row_sets)))
}
// ANCHOR_END: post_query
// ANCHOR: execute_query_with_variables
// ANCHOR: execute_query_with_variables_signature
fn execute_query_with_variables(
    collection: &str,
    arguments: &BTreeMap<String, models::Argument>,
    collection_relationships: &BTreeMap<String, models::Relationship>,
    query: &models::Query,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
) -> Result<models::RowSet> {
    // ANCHOR_END: execute_query_with_variables_signature
    let mut argument_values = BTreeMap::new();

    for (argument_name, argument_value) in arguments.iter() {
        if argument_values
            .insert(
                argument_name.clone(),
                eval_argument(variables, argument_value)?,
            )
            .is_some()
        {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(models::ErrorResponse {
                    message: "duplicate argument names".into(),
                    details: serde_json::Value::Null,
                }),
            ));
        }
    }

    let collection = get_collection_by_name(collection, &argument_values, state)?;

    execute_query(
        collection_relationships,
        variables,
        state,
        query,
        Root::CurrentRow,
        collection,
    )
}
// ANCHOR_END: execute_query_with_variables
// ANCHOR: get_collection_by_name
fn get_collection_by_name(
    collection_name: &str,
    arguments: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
) -> Result<Vec<Row>> {
    match collection_name {
        "articles" => Ok(state.articles.values().cloned().collect()),
        "authors" => Ok(state.authors.values().cloned().collect()),
        "universities" => Ok(state.universities.values().cloned().collect()),
        "articles_by_author" => {
            let author_id = arguments.get("author_id").ok_or((
                StatusCode::BAD_REQUEST,
                Json(models::ErrorResponse {
                    message: "missing argument author_id".into(),
                    details: serde_json::Value::Null,
                }),
            ))?;
            let author_id_int = author_id.as_i64().ok_or((
                StatusCode::BAD_REQUEST,
                Json(models::ErrorResponse {
                    message: "author_id must be a string".into(),
                    details: serde_json::Value::Null,
                }),
            ))?;

            let mut articles_by_author = vec![];

            for (_id, article) in state.articles.iter() {
                let article_author_id = article.get("author_id").ok_or((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(models::ErrorResponse {
                        message: "author_id not found".into(),
                        details: serde_json::Value::Null,
                    }),
                ))?;
                let article_author_id_int = article_author_id.as_i64().ok_or((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(models::ErrorResponse {
                        message: " ".into(),
                        details: serde_json::Value::Null,
                    }),
                ))?;
                if article_author_id_int == author_id_int {
                    articles_by_author.push(article.clone())
                }
            }

            Ok(articles_by_author)
        }
        "latest_article_id" => {
            let latest_id = state
                .articles
                .iter()
                .filter_map(|(_id, a)| a.get("id").and_then(|v| v.as_i64()))
                .max();
            let latest_id_value = serde_json::to_value(latest_id).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(models::ErrorResponse {
                        message: " ".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })?;
            Ok(vec![BTreeMap::from_iter([(
                "__value".into(),
                latest_id_value,
            )])])
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(models::ErrorResponse {
                message: "invalid collection name".into(),
                details: serde_json::Value::Null,
            }),
        )),
    }
}
// ANCHOR_END: get_collection_by_name
/// ANCHOR: Root
enum Root<'a> {
    /// References to the root collection actually
    /// refer to the current row, because the path to
    /// the nearest enclosing [`models::Query`] does not pass
    /// an [`models::Expression::Exists`] node.
    CurrentRow,
    /// References to the root collection refer to the
    /// explicitly-identified row, which is the row
    /// being evaluated in the context of the nearest enclosing
    /// [`models::Query`].
    ExplicitRow(&'a Row),
}
/// ANCHOR_END: Root
// ANCHOR: execute_query
// ANCHOR: execute_query_signature
fn execute_query(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    query: &models::Query,
    root: Root,
    collection: Vec<Row>,
) -> Result<models::RowSet> {
    // ANCHOR_END: execute_query_signature
    // ANCHOR: execute_query_sort
    let sorted = sort(
        collection_relationships,
        variables,
        state,
        collection,
        &query.order_by,
    )?;
    // ANCHOR_END: execute_query_sort
    // ANCHOR: execute_query_filter
    let filtered: Vec<Row> = (match &query.predicate {
        None => Ok(sorted),
        Some(expr) => {
            let mut filtered: Vec<Row> = vec![];
            for item in sorted.into_iter() {
                let root = match root {
                    Root::CurrentRow => &item,
                    Root::ExplicitRow(root) => root,
                };
                if eval_expression(
                    collection_relationships,
                    variables,
                    state,
                    expr,
                    root,
                    &item,
                )? {
                    filtered.push(item);
                }
            }
            Ok(filtered)
        }
    })?;
    // ANCHOR_END: execute_query_filter
    // ANCHOR: execute_query_paginate
    let paginated: Vec<Row> = paginate(filtered.into_iter(), query.limit, query.offset);
    // ANCHOR_END: execute_query_paginate
    // ANCHOR: execute_query_aggregates
    let aggregates = query
        .aggregates
        .as_ref()
        .map(|aggregates| {
            let mut row: IndexMap<String, serde_json::Value> = IndexMap::new();
            for (aggregate_name, aggregate) in aggregates.iter() {
                row.insert(
                    aggregate_name.clone(),
                    eval_aggregate(aggregate, &paginated)?,
                );
            }
            Ok(row)
        })
        .transpose()?;
    // ANCHOR_END: execute_query_aggregates
    // ANCHOR: execute_query_fields
    let rows = query
        .fields
        .as_ref()
        .map(|fields| {
            let mut rows: Vec<IndexMap<String, models::RowFieldValue>> = vec![];
            for item in paginated.iter() {
                let row = eval_row(fields, collection_relationships, variables, state, item)?;
                rows.push(row)
            }
            Ok(rows)
        })
        .transpose()?;
    // ANCHOR_END: execute_query_fields
    // ANCHOR: execute_query_rowset
    Ok(models::RowSet { aggregates, rows })
    // ANCHOR_END: execute_query_rowset
}
// ANCHOR_END: execute_query
// ANCHOR: eval_row
fn eval_row(
    fields: &IndexMap<String, models::Field>,
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, Value>,
    state: &AppState,
    item: &BTreeMap<String, Value>,
) -> Result<IndexMap<String, models::RowFieldValue>> {
    let mut row = IndexMap::new();
    for (field_name, field) in fields.iter() {
        row.insert(
            field_name.clone(),
            eval_field(collection_relationships, variables, state, field, item)?,
        );
    }
    Ok(row)
}
// ANCHOR_END: eval_row
// ANCHOR: eval_aggregate
fn eval_aggregate(
    aggregate: &models::Aggregate,
    paginated: &Vec<BTreeMap<String, serde_json::Value>>,
) -> Result<serde_json::Value> {
    match aggregate {
        models::Aggregate::StarCount {} => Ok(serde_json::Value::from(paginated.len())),
        models::Aggregate::ColumnCount { column, distinct } => {
            let values = paginated
                .iter()
                .map(|row| {
                    row.get(column).ok_or((
                        StatusCode::BAD_REQUEST,
                        Json(models::ErrorResponse {
                            message: "invalid column name".into(),
                            details: serde_json::Value::Null,
                        }),
                    ))
                })
                .collect::<Result<Vec<_>>>()?;

            let non_null_values = values.iter().filter(|value| !value.is_null());

            let agg_value = if *distinct {
                non_null_values
                    .map(|value| {
                        serde_json::to_string(value).map_err(|_| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(models::ErrorResponse {
                                    message: "unable to encode value".into(),
                                    details: serde_json::Value::Null,
                                }),
                            )
                        })
                    })
                    .collect::<Result<HashSet<_>>>()?
                    .len()
            } else {
                non_null_values.count()
            };
            serde_json::to_value(agg_value).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(models::ErrorResponse {
                        message: " ".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })
        }
        models::Aggregate::SingleColumn { column, function } => {
            let values = paginated
                .iter()
                .map(|row| {
                    row.get(column).ok_or((
                        StatusCode::BAD_REQUEST,
                        Json(models::ErrorResponse {
                            message: "invalid column name".into(),
                            details: serde_json::Value::Null,
                        }),
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            eval_aggregate_function(function, values)
        }
    }
}
// ANCHOR_END: eval_aggregate
// ANCHOR: eval_aggregate_function
fn eval_aggregate_function(
    function: &str,
    values: Vec<&serde_json::Value>,
) -> Result<serde_json::Value> {
    let int_values = values
        .iter()
        .map(|value| {
            value.as_i64().ok_or((
                StatusCode::BAD_REQUEST,
                Json(models::ErrorResponse {
                    message: "column is not an integer".into(),
                    details: serde_json::Value::Null,
                }),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let agg_value = match function {
        "min" => Ok(int_values.iter().min()),
        "max" => Ok(int_values.iter().max()),
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(models::ErrorResponse {
                message: "invalid aggregation function".into(),
                details: serde_json::Value::Null,
            }),
        )),
    }?;
    serde_json::to_value(agg_value).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(models::ErrorResponse {
                message: " ".into(),
                details: serde_json::Value::Null,
            }),
        )
    })
}
// ANCHOR_END: eval_aggregate_function
// ANCHOR: sort
fn sort(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    collection: Vec<Row>,
    order_by: &Option<models::OrderBy>,
) -> Result<Vec<Row>> {
    match order_by {
        None => Ok(collection),
        Some(order_by) => {
            let mut copy = vec![];
            for item_to_insert in collection.into_iter() {
                let mut index = 0;
                for other in copy.iter() {
                    if let Ordering::Greater = eval_order_by(
                        collection_relationships,
                        variables,
                        state,
                        order_by,
                        other,
                        &item_to_insert,
                    )? {
                        break;
                    } else {
                        index += 1;
                    }
                }
                copy.insert(index, item_to_insert);
            }
            Ok(copy)
        }
    }
}
// ANCHOR_END: sort
// ANCHOR: paginate
fn paginate<I: Iterator<Item = Row>>(
    collection: I,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Vec<Row> {
    let start = offset.unwrap_or(0).try_into().unwrap();
    match limit {
        Some(n) => collection.skip(start).take(n.try_into().unwrap()).collect(),
        None => collection.skip(start).collect(),
    }
}
// ANCHOR_END: paginate
// ANCHOR: eval_order_by
fn eval_order_by(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    order_by: &models::OrderBy,
    t1: &Row,
    t2: &Row,
) -> Result<Ordering> {
    let mut result = Ordering::Equal;

    for element in order_by.elements.iter() {
        let v1 = eval_order_by_element(collection_relationships, variables, state, element, t1)?;
        let v2 = eval_order_by_element(collection_relationships, variables, state, element, t2)?;
        let x = match element.order_direction {
            models::OrderDirection::Asc => compare(v1, v2)?,
            models::OrderDirection::Desc => compare(v2, v1)?,
        };
        result = result.then(x);
    }

    Ok(result)
}
// ANCHOR_END: eval_order_by
// ANCHOR: compare
fn compare(v1: serde_json::Value, v2: serde_json::Value) -> Result<Ordering> {
    match (v1, v2) {
        (serde_json::Value::Null, serde_json::Value::Null) => Ok(Ordering::Equal),
        (serde_json::Value::Null, _) => Ok(Ordering::Less),
        (_, serde_json::Value::Null) => Ok(Ordering::Greater),

        (serde_json::Value::Bool(b1), serde_json::Value::Bool(b2)) => Ok(b1.cmp(&b2)),
        (serde_json::Value::Number(n1), serde_json::Value::Number(n2)) => {
            if n1.as_f64().unwrap() < n2.as_f64().unwrap() {
                Ok(Ordering::Less)
            } else {
                Ok(Ordering::Greater)
            }
        }
        (serde_json::Value::String(s1), serde_json::Value::String(s2)) => Ok(s1.cmp(&s2)),
        _ => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(models::ErrorResponse {
                message: "cannot compare values".into(),
                details: serde_json::Value::Null,
            }),
        )),
    }
}
// ANCHOR_END: compare
// ANCHOR: eval_order_by_element
fn eval_order_by_element(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    element: &models::OrderByElement,
    item: &Row,
) -> Result<serde_json::Value> {
    match element.target.clone() {
        models::OrderByTarget::Column { name, path } => {
            eval_order_by_column(collection_relationships, variables, state, item, path, name)
        }
        models::OrderByTarget::SingleColumnAggregate {
            column,
            function,
            path,
        } => eval_order_by_single_column_aggregate(
            collection_relationships,
            variables,
            state,
            item,
            path,
            column,
            function,
        ),
        models::OrderByTarget::StarCountAggregate { path } => eval_order_by_star_count_aggregate(
            collection_relationships,
            variables,
            state,
            item,
            path,
        ),
    }
}
// ANCHOR_END: eval_order_by_element
// ANCHOR: eval_order_by_star_count_aggregate
fn eval_order_by_star_count_aggregate(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    item: &BTreeMap<String, serde_json::Value>,
    path: Vec<models::PathElement>,
) -> Result<serde_json::Value> {
    let rows: Vec<Row> = eval_path(collection_relationships, variables, state, &path, item)?;
    Ok(rows.len().into())
}
// ANCHOR_END: eval_order_by_star_count_aggregate
// ANCHOR: eval_order_by_single_column_aggregate
fn eval_order_by_single_column_aggregate(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    item: &BTreeMap<String, serde_json::Value>,
    path: Vec<models::PathElement>,
    column: String,
    function: String,
) -> Result<serde_json::Value> {
    let rows: Vec<Row> = eval_path(collection_relationships, variables, state, &path, item)?;
    let values = rows
        .iter()
        .map(|row| {
            row.get(column.as_str()).ok_or((
                StatusCode::BAD_REQUEST,
                Json(models::ErrorResponse {
                    message: "invalid column name".into(),
                    details: serde_json::Value::Null,
                }),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    eval_aggregate_function(&function, values)
}
// ANCHOR_END: eval_order_by_single_column_aggregate
// ANCHOR: eval_order_by_column
fn eval_order_by_column(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    item: &BTreeMap<String, serde_json::Value>,
    path: Vec<models::PathElement>,
    name: String,
) -> Result<serde_json::Value> {
    let rows: Vec<Row> = eval_path(collection_relationships, variables, state, &path, item)?;
    if rows.len() > 1 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(models::ErrorResponse {
                message: " ".into(),
                details: serde_json::Value::Null,
            }),
        ));
    }
    match rows.first() {
        Some(row) => eval_column(row, name.as_str()),
        None => Ok(serde_json::Value::Null),
    }
}
// ANCHOR_END: eval_order_by_column
// ANCHOR: eval_path
fn eval_path(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    path: &[models::PathElement],
    item: &Row,
) -> Result<Vec<Row>> {
    let mut result: Vec<Row> = vec![item.clone()];

    for path_element in path.iter() {
        let relationship_name = path_element.relationship.as_str();
        let relationship = collection_relationships.get(relationship_name).ok_or((
            StatusCode::BAD_REQUEST,
            Json(models::ErrorResponse {
                message: "invalid relationship name in path".into(),
                details: serde_json::Value::Null,
            }),
        ))?;
        result = eval_path_element(
            collection_relationships,
            variables,
            state,
            relationship,
            &path_element.arguments,
            &result,
            &path_element.predicate,
        )?;
    }

    Ok(result)
}
// ANCHOR_END: eval_path
// ANCHOR: eval_path_element
fn eval_path_element(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    relationship: &models::Relationship,
    arguments: &BTreeMap<String, models::RelationshipArgument>,
    source: &[Row],
    predicate: &models::Expression,
) -> Result<Vec<Row>> {
    let mut matching_rows: Vec<Row> = vec![];

    // Note: Join strategy
    //
    // Rows can be related in two ways: 1) via a column mapping, and
    // 2) via collection arguments. Because collection arguments can be computed
    // using the columns on the source side of a relationship, in general
    // we need to compute the target collection once for each source row.
    // This join strategy can result in some target rows appearing in the
    // resulting row set more than once, if two source rows are both related
    // to the same target row.
    //
    // In practice, this is not an issue, either because a) the relationship
    // is computed in the course of evaluating a predicate, and all predicates are
    // implicitly or explicitly existentially quantified, or b) if the
    // relationship is computed in the course of evaluating an ordering, the path
    // should consist of all object relationships, and possibly terminated by a
    // single array relationship, so there should be no double counting.

    for src_row in source.iter() {
        let mut all_arguments = BTreeMap::new();

        for (argument_name, argument_value) in relationship.arguments.iter() {
            if all_arguments
                .insert(
                    argument_name.clone(),
                    eval_relationship_argument(variables, src_row, argument_value)?,
                )
                .is_some()
            {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "duplicate argument names".into(),
                        details: serde_json::Value::Null,
                    }),
                ));
            }
        }

        for (argument_name, argument_value) in arguments.iter() {
            if all_arguments
                .insert(
                    argument_name.clone(),
                    eval_relationship_argument(variables, src_row, argument_value)?,
                )
                .is_some()
            {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "duplicate argument names".into(),
                        details: serde_json::Value::Null,
                    }),
                ));
            }
        }

        let target = get_collection_by_name(
            relationship.target_collection.as_str(),
            &all_arguments,
            state,
        )?;

        for tgt_row in target.iter() {
            if eval_column_mapping(relationship, src_row, tgt_row)?
                && eval_expression(
                    collection_relationships,
                    variables,
                    state,
                    predicate,
                    tgt_row,
                    tgt_row,
                )?
            {
                matching_rows.push(tgt_row.clone());
            }
        }
    }

    Ok(matching_rows)
}
// ANCHOR_END: eval_path_element
// ANCHOR: eval_argument
fn eval_argument(
    variables: &BTreeMap<String, serde_json::Value>,
    argument: &models::Argument,
) -> Result<serde_json::Value> {
    match argument {
        models::Argument::Variable { name } => {
            let value = variables
                .get(name.as_str())
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "invalid variable name".into(),
                        details: serde_json::Value::Null,
                    }),
                ))
                .cloned()?;
            Ok(value)
        }
        models::Argument::Literal { value } => Ok(value.clone()),
    }
}
// ANCHOR_END: eval_argument
// ANCHOR: eval_relationship_argument
fn eval_relationship_argument(
    variables: &BTreeMap<String, serde_json::Value>,
    row: &Row,
    argument: &models::RelationshipArgument,
) -> Result<serde_json::Value> {
    match argument {
        models::RelationshipArgument::Variable { name } => {
            let value = variables
                .get(name.as_str())
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "invalid variable name".into(),
                        details: serde_json::Value::Null,
                    }),
                ))
                .cloned()?;
            Ok(value)
        }
        models::RelationshipArgument::Literal { value } => Ok(value.clone()),
        models::RelationshipArgument::Column { name } => eval_column(row, name),
    }
}
// ANCHOR_END: eval_relationship_argument
// ANCHOR: eval_expression
// ANCHOR: eval_expression_signature
fn eval_expression(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    expr: &models::Expression,
    root: &Row,
    item: &Row,
) -> Result<bool> {
    // ANCHOR_END: eval_expression_signature
    // ANCHOR: eval_expression_logical
    match expr {
        models::Expression::And { expressions } => {
            for expr in expressions.iter() {
                if !eval_expression(collection_relationships, variables, state, expr, root, item)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        models::Expression::Or { expressions } => {
            for expr in expressions.iter() {
                if eval_expression(collection_relationships, variables, state, expr, root, item)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        models::Expression::Not { expression } => {
            let b = eval_expression(
                collection_relationships,
                variables,
                state,
                expression,
                root,
                item,
            )?;
            Ok(!b)
        }
        // ANCHOR_END: eval_expression_logical
        // ANCHOR: eval_expression_unary_operators
        models::Expression::UnaryComparisonOperator { column, operator } => match operator {
            models::UnaryComparisonOperator::IsNull => {
                let vals = eval_comparison_target(
                    collection_relationships,
                    variables,
                    state,
                    column,
                    root,
                    item,
                )?;
                Ok(vals.iter().any(|val| val.is_null()))
            }
        },
        // ANCHOR_END: eval_expression_unary_operators
        // ANCHOR: eval_expression_binary_operators
        models::Expression::BinaryComparisonOperator {
            column,
            operator,
            value,
        } => match operator {
            models::BinaryComparisonOperator::Equal => {
                let left_vals = eval_comparison_target(
                    collection_relationships,
                    variables,
                    state,
                    column,
                    root,
                    item,
                )?;
                let right_vals = eval_comparison_value(
                    collection_relationships,
                    variables,
                    state,
                    value,
                    root,
                    item,
                )?;
                for left_val in left_vals.iter() {
                    for right_val in right_vals.iter() {
                        if left_val == right_val {
                            return Ok(true);
                        }
                    }
                }

                Ok(false)
            }
            // ANCHOR_END: eval_expression_binary_operators
            // ANCHOR: eval_expression_custom_binary_operators
            models::BinaryComparisonOperator::Other { name } => match name.as_str() {
                "like" => {
                    let column_vals = eval_comparison_target(
                        collection_relationships,
                        variables,
                        state,
                        column,
                        root,
                        item,
                    )?;
                    let regex_vals = eval_comparison_value(
                        collection_relationships,
                        variables,
                        state,
                        value,
                        root,
                        item,
                    )?;
                    for column_val in column_vals.iter() {
                        for regex_val in regex_vals.iter() {
                            let column_str = column_val.as_str().ok_or((
                                StatusCode::BAD_REQUEST,
                                Json(models::ErrorResponse {
                                    message: "column is not a string".into(),
                                    details: serde_json::Value::Null,
                                }),
                            ))?;
                            let regex_str = regex_val.as_str().ok_or((
                                StatusCode::BAD_REQUEST,
                                Json(models::ErrorResponse {
                                    message: " ".into(),
                                    details: serde_json::Value::Null,
                                }),
                            ))?;
                            let regex = Regex::new(regex_str).map_err(|_| {
                                (
                                    StatusCode::BAD_REQUEST,
                                    Json(models::ErrorResponse {
                                        message: "invalid regular expression".into(),
                                        details: serde_json::Value::Null,
                                    }),
                                )
                            })?;
                            if regex.is_match(column_str) {
                                return Ok(true);
                            }
                        }
                    }
                    Ok(false)
                }
                _ => Err((
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: " ".into(),
                        details: serde_json::Value::Null,
                    }),
                )),
            },
            // ANCHOR_END: eval_expression_custom_binary_operators
        },
        // ANCHOR: eval_expression_binary_array_operators
        models::Expression::BinaryArrayComparisonOperator {
            column,
            operator,
            values,
        } => match operator {
            models::BinaryArrayComparisonOperator::In => {
                let left_vals = eval_comparison_target(
                    collection_relationships,
                    variables,
                    state,
                    column,
                    root,
                    item,
                )?;

                for comparison_value in values.iter() {
                    let right_vals = eval_comparison_value(
                        collection_relationships,
                        variables,
                        state,
                        comparison_value,
                        root,
                        item,
                    )?;
                    for left_val in left_vals.iter() {
                        for right_val in right_vals.iter() {
                            if left_val == right_val {
                                return Ok(true);
                            }
                        }
                    }
                }
                Ok(false)
            }
        },
        // ANCHOR_END: eval_expression_binary_array_operators
        // ANCHOR: eval_expression_exists
        models::Expression::Exists {
            in_collection,
            predicate,
        } => {
            let query = models::Query {
                aggregates: None,
                fields: Some(IndexMap::new()),
                limit: None,
                offset: None,
                order_by: None,
                predicate: Some(*predicate.clone()),
            };
            let collection = eval_in_collection(
                collection_relationships,
                item,
                variables,
                state,
                in_collection,
            )?;
            let row_set = execute_query(
                collection_relationships,
                variables,
                state,
                &query,
                Root::ExplicitRow(root),
                collection,
            )?;
            let rows: Vec<IndexMap<_, _>> = row_set.rows.ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(models::ErrorResponse {
                    message: " ".into(),
                    details: serde_json::Value::Null,
                }),
            ))?;
            Ok(!rows.is_empty())
        } // ANCHOR_END: eval_expression_exists
    }
}
// ANCHOR_END: eval_expression
// ANCHOR: eval_in_collection
fn eval_in_collection(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    item: &BTreeMap<String, serde_json::Value>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    in_collection: &models::ExistsInCollection,
) -> Result<Vec<Row>> {
    match in_collection {
        models::ExistsInCollection::Related {
            relationship,
            arguments,
        } => {
            let relationship = collection_relationships.get(relationship.as_str()).ok_or((
                StatusCode::BAD_REQUEST,
                Json(models::ErrorResponse {
                    message: " ".into(),
                    details: serde_json::Value::Null,
                }),
            ))?;
            let source = vec![item.clone()];
            eval_path_element(
                collection_relationships,
                variables,
                state,
                relationship,
                arguments,
                &source,
                &models::Expression::And {
                    expressions: vec![],
                },
            )
        }
        models::ExistsInCollection::Unrelated {
            collection,
            arguments,
        } => {
            let arguments = arguments
                .iter()
                .map(|(k, v)| Ok((k.clone(), eval_relationship_argument(variables, item, v)?)))
                .collect::<Result<BTreeMap<_, _>>>()?;

            get_collection_by_name(collection.as_str(), &arguments, state)
        }
    }
}
// ANCHOR_END: eval_in_collection
// ANCHOR: eval_comparison_target
fn eval_comparison_target(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    target: &models::ComparisonTarget,
    root: &Row,
    item: &Row,
) -> Result<Vec<serde_json::Value>> {
    match target {
        models::ComparisonTarget::Column { name, path } => {
            let rows = eval_path(collection_relationships, variables, state, path, item)?;
            let mut values = vec![];
            for row in rows.iter() {
                let value = eval_column(row, name.as_str())?;
                values.push(value);
            }
            Ok(values)
        }
        models::ComparisonTarget::RootCollectionColumn { name } => {
            let value = eval_column(root, name.as_str())?;
            Ok(vec![value])
        }
    }
}
// ANCHOR_END: eval_comparison_target
// ANCHOR: eval_column
fn eval_column(row: &Row, column_name: &str) -> Result<serde_json::Value> {
    row.get(column_name).cloned().ok_or((
        StatusCode::BAD_REQUEST,
        Json(models::ErrorResponse {
            message: "invalid column name".into(),
            details: serde_json::Value::Null,
        }),
    ))
}
// ANCHOR_END: eval_column
// ANCHOR: eval_comparison_value
fn eval_comparison_value(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    comparison_value: &models::ComparisonValue,
    root: &Row,
    item: &Row,
) -> Result<Vec<serde_json::Value>> {
    match comparison_value {
        models::ComparisonValue::Column { column } => eval_comparison_target(
            collection_relationships,
            variables,
            state,
            column,
            root,
            item,
        ),
        models::ComparisonValue::Scalar { value } => Ok(vec![value.clone()]),
        models::ComparisonValue::Variable { name } => {
            let value = variables
                .get(name.as_str())
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "invalid variable name".into(),
                        details: serde_json::Value::Null,
                    }),
                ))
                .cloned()?;
            Ok(vec![value])
        }
    }
}
// ANCHOR_END: eval_comparison_value
// ANCHOR: eval_nested_field
fn eval_nested_field(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    value: Value,
    nested_field: &models::NestedField,
) -> Result<models::RowFieldValue> {
    match nested_field {
        models::NestedField::Object(models::NestedObject { fields }) => {
            let full_row: Row = serde_json::from_value(value).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "Expected object".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })?;
            let row = eval_row(
                fields,
                collection_relationships,
                variables,
                state,
                &full_row,
            )?;
            Ok(models::RowFieldValue(serde_json::to_value(row).map_err(
                |_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(models::ErrorResponse {
                            message: "Cannot encode rowset".into(),
                            details: serde_json::Value::Null,
                        }),
                    )
                },
            )?))
        }
        models::NestedField::Array(models::NestedArray { fields }) => {
            let array: Vec<Value> = serde_json::from_value(value).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "Expected array".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })?;
            let result_array = match fields.deref() {
                None => array
                    .into_iter()
                    .map(models::RowFieldValue)
                    .collect::<Vec<_>>(),
                Some(field) => array
                    .into_iter()
                    .map(|value| {
                        eval_nested_field(collection_relationships, variables, state, value, field)
                    })
                    .collect::<Result<Vec<_>>>()?,
            };
            Ok(models::RowFieldValue(
                serde_json::to_value(result_array).map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(models::ErrorResponse {
                            message: "Cannot encode rowset".into(),
                            details: serde_json::Value::Null,
                        }),
                    )
                })?,
            ))
        }
    }
}
// ANCHOR_END: eval_nested_field
// ANCHOR: eval_field
fn eval_field(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    field: &models::Field,
    item: &Row,
) -> Result<models::RowFieldValue> {
    match field {
        models::Field::Column { column, fields } => {
            let col_val = eval_column(item, column.as_str())?;
            match fields {
                None => Ok(models::RowFieldValue(col_val)),
                Some(nested_field) => eval_nested_field(
                    collection_relationships,
                    variables,
                    state,
                    col_val,
                    nested_field,
                ),
            }
        }
        models::Field::Relationship {
            relationship,
            arguments,
            query,
        } => {
            let relationship = collection_relationships.get(relationship.as_str()).ok_or((
                StatusCode::BAD_REQUEST,
                Json(models::ErrorResponse {
                    message: " ".into(),
                    details: serde_json::Value::Null,
                }),
            ))?;
            let source = vec![item.clone()];
            let collection = eval_path_element(
                collection_relationships,
                variables,
                state,
                relationship,
                arguments,
                &source,
                &models::Expression::And {
                    expressions: vec![],
                },
            )?;
            let rows = execute_query(
                collection_relationships,
                variables,
                state,
                query,
                Root::CurrentRow,
                collection,
            )?;
            let rows_json = serde_json::to_value(rows).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(models::ErrorResponse {
                        message: "cannot encode rowset".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })?;
            Ok(models::RowFieldValue(rows_json))
        }
    }
}
// ANCHOR_END: eval_field
// ANCHOR: explain
async fn post_explain(
    Json(_request): Json<models::QueryRequest>,
) -> Result<Json<models::ExplainResponse>> {
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(models::ErrorResponse {
            message: "explain is not supported".into(),
            details: serde_json::Value::Null,
        }),
    ))
}
// ANCHOR_END: explain
// ANCHOR: post_mutation_signature
async fn post_mutation(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(request): Json<models::MutationRequest>,
) -> Result<Json<models::MutationResponse>> {
    // ANCHOR_END: post_mutation_signature
    // ANCHOR: post_mutation
    let mut state = state.lock().await;

    let mut operation_results = vec![];

    for operation in request.operations.iter() {
        let operation_result =
            execute_mutation_operation(&mut state, &request.collection_relationships, operation)?;
        operation_results.push(operation_result);
    }

    Ok(Json(models::MutationResponse { operation_results }))
}
// ANCHOR_END: post_mutation
// ANCHOR: execute_mutation_operation
fn execute_mutation_operation(
    state: &mut AppState,
    collection_relationships: &BTreeMap<String, models::Relationship>,
    operation: &models::MutationOperation,
) -> Result<models::MutationOperationResults> {
    match operation {
        models::MutationOperation::Procedure {
            name,
            arguments,
            fields,
        } => execute_procedure(state, name, arguments, fields, collection_relationships),
    }
}
// ANCHOR_END: execute_mutation_operation
// ANCHOR: execute_procedure_signature
fn execute_procedure(
    state: &mut AppState,
    name: &str,
    arguments: &BTreeMap<String, serde_json::Value>,
    fields: &Option<IndexMap<String, models::Field>>,
    collection_relationships: &BTreeMap<String, models::Relationship>,
) -> std::result::Result<models::MutationOperationResults, (StatusCode, Json<models::ErrorResponse>)>
// ANCHOR_END: execute_procedure_signature
// ANCHOR: execute_procedure_signature_impl
{
    match name {
        "upsert_article" => {
            execute_upsert_article(state, arguments, fields, collection_relationships)
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(models::ErrorResponse {
                message: "unknown procedure".into(),
                details: serde_json::Value::Null,
            }),
        )),
    }
}
// ANCHOR_END: execute_procedure_signature_impl
// ANCHOR: execute_upsert_article
fn execute_upsert_article(
    state: &mut AppState,
    arguments: &BTreeMap<String, serde_json::Value>,
    fields: &Option<IndexMap<String, models::Field>>,
    collection_relationships: &BTreeMap<String, models::Relationship>,
) -> std::result::Result<models::MutationOperationResults, (StatusCode, Json<models::ErrorResponse>)>
{
    let article = arguments.get("article").ok_or((
        StatusCode::BAD_REQUEST,
        Json(models::ErrorResponse {
            message: " ".into(),
            details: serde_json::Value::Null,
        }),
    ))?;
    let article_obj = article.as_object().ok_or((
        StatusCode::BAD_REQUEST,
        Json(models::ErrorResponse {
            message: " ".into(),
            details: serde_json::Value::Null,
        }),
    ))?;
    let id = article_obj.get("id").ok_or((
        StatusCode::BAD_REQUEST,
        Json(models::ErrorResponse {
            message: " ".into(),
            details: serde_json::Value::Null,
        }),
    ))?;
    let id_int = id.as_i64().ok_or((
        StatusCode::BAD_REQUEST,
        Json(models::ErrorResponse {
            message: " ".into(),
            details: serde_json::Value::Null,
        }),
    ))?;
    let new_row = BTreeMap::from_iter(article_obj.iter().map(|(k, v)| (k.clone(), v.clone())));
    let old_row = state.articles.insert(id_int, new_row);
    let returning = old_row
        .map(|old_row| {
            let mut row = IndexMap::new();
            for fields in fields.iter() {
                for (field_name, field) in fields.iter() {
                    row.insert(
                        field_name.clone(),
                        eval_field(
                            collection_relationships,
                            &BTreeMap::new(),
                            state,
                            field,
                            &old_row,
                        )?,
                    );
                }
            }
            Ok(row)
        })
        .transpose()?;
    Ok(models::MutationOperationResults {
        affected_rows: 1,
        returning: Some(vec![IndexMap::from_iter([(
            "__value".into(),
            models::RowFieldValue(serde_json::to_value(returning).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(models::ErrorResponse {
                        message: "cannot encode response".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })?),
        )])]),
    })
}
// ANCHOR_END: execute_upsert_article

fn eval_column_mapping(
    relationship: &models::Relationship,
    src_row: &Row,
    tgt_row: &Row,
) -> Result<bool> {
    for (src_column, tgt_column) in relationship.column_mapping.iter() {
        let src_value = eval_column(src_row, src_column)?;
        let tgt_value = eval_column(tgt_row, tgt_column)?;
        if src_value != tgt_value {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use axum::{extract::State, Json};
    use goldenfile::Mint;
    use ndc_client::models;
    use ndc_test::{test_connector, Connector, Error, TestConfiguration};
    use std::{
        fs::{self, File},
        io::Write,
        path::PathBuf,
        sync::Arc,
    };
    use tokio::sync::Mutex;

    use crate::{get_capabilities, get_schema, init_app_state, post_mutation, post_query};

    #[test]
    fn test_capabilities() {
        tokio_test::block_on(async {
            let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

            let mut mint = Mint::new(&test_dir);

            let expected_path = PathBuf::from_iter(["capabilities", "expected.json"]);

            let response = crate::get_capabilities().await;

            let mut expected = mint.new_goldenfile(expected_path).unwrap();

            let response_json = serde_json::to_string_pretty(&response.0).unwrap();

            write!(expected, "{}", response_json).unwrap();

            // Test roundtrip
            assert_eq!(
                response.0,
                serde_json::from_str(response_json.as_str()).unwrap()
            );
        });
    }

    #[test]
    fn test_schema() {
        tokio_test::block_on(async {
            let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

            let mut mint = Mint::new(&test_dir);

            let expected_path = PathBuf::from_iter(["schema", "expected.json"]);

            let response = crate::get_schema().await;

            let mut expected = mint.new_goldenfile(expected_path).unwrap();

            write!(
                expected,
                "{}",
                serde_json::to_string_pretty(&response.0).unwrap()
            )
            .unwrap();
        });
    }

    #[test]
    fn test_query() {
        tokio_test::block_on(async {
            let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

            let mut mint = Mint::new(&test_dir);

            for input_file in fs::read_dir(test_dir.join("query")).unwrap() {
                let entry = input_file.unwrap();
                let request = {
                    let path = entry.path();
                    assert!(path.is_dir());
                    let req_path = path.join("request.json");
                    let req_file = File::open(req_path).unwrap();
                    serde_json::from_reader::<_, models::QueryRequest>(req_file).unwrap()
                };

                let expected_path = {
                    let path = entry.path();
                    let test_name = path.file_name().unwrap().to_str().unwrap();
                    PathBuf::from_iter(["query", test_name, "expected.json"])
                };

                let state = Arc::new(Mutex::new(crate::init_app_state()));
                let response = crate::post_query(State(state), Json(request))
                    .await
                    .unwrap();

                let mut expected = mint.new_goldenfile(expected_path).unwrap();

                write!(
                    expected,
                    "{}",
                    serde_json::to_string_pretty(&response.0).unwrap()
                )
                .unwrap();
            }
        });
    }

    #[test]
    fn test_mutation() {
        tokio_test::block_on(async {
            let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

            let mut mint = Mint::new(&test_dir);

            for input_file in fs::read_dir(test_dir.join("mutation")).unwrap() {
                let entry = input_file.unwrap();
                let request = {
                    let path = entry.path();
                    assert!(path.is_dir());
                    let req_path = path.join("request.json");
                    let req_file = File::open(req_path).unwrap();
                    serde_json::from_reader::<_, models::MutationRequest>(req_file).unwrap()
                };

                let expected_path = {
                    let path = entry.path();
                    let test_name = path.file_name().unwrap().to_str().unwrap();
                    PathBuf::from_iter(["mutation", test_name, "expected.json"])
                };

                let state = Arc::new(Mutex::new(crate::init_app_state()));
                let response = crate::post_mutation(State(state), Json(request))
                    .await
                    .unwrap();

                let mut expected = mint.new_goldenfile(expected_path).unwrap();

                write!(
                    expected,
                    "{}",
                    serde_json::to_string_pretty(&response.0).unwrap()
                )
                .unwrap();
            }
        });
    }

    struct Reference {
        state: Arc<Mutex<crate::AppState>>,
    }

    #[async_trait]
    impl Connector for Reference {
        async fn get_capabilities(&self) -> Result<models::CapabilitiesResponse, Error> {
            Ok(get_capabilities().await.0)
        }

        async fn get_schema(&self) -> Result<models::SchemaResponse, Error> {
            Ok(get_schema().await.0)
        }

        async fn query(
            &self,
            request: models::QueryRequest,
        ) -> Result<models::QueryResponse, Error> {
            Ok(post_query(State(self.state.clone()), Json(request))
                .await
                .map_err(|(_, Json(err))| Error::ConnectorError(err))?
                .0)
        }

        async fn mutation(
            &self,
            request: models::MutationRequest,
        ) -> Result<models::MutationResponse, Error> {
            Ok(post_mutation(State(self.state.clone()), Json(request))
                .await
                .map_err(|(_, Json(err))| Error::ConnectorError(err))?
                .0)
        }
    }

    #[test]
    fn test_ndc_test() {
        tokio_test::block_on(async {
            let configuration = TestConfiguration {
                seed: None,
                snapshots_dir: None,
            };
            let connector = Reference {
                state: Arc::new(Mutex::new(init_app_state())),
            };
            let results = test_connector(&configuration, &connector).await;
            assert!(results.failures.is_empty());
        });
    }
}
