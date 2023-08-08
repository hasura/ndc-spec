use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashSet},
    error::Error,
    sync::Arc,
};

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use csv;
use indexmap::IndexMap;
use ndc_client::models;
use prometheus::{Encoder, IntCounter, IntGauge, Opts, Registry, TextEncoder};
use regex::Regex;
use tokio::sync::Mutex;

// ANCHOR: row-type
type Row = BTreeMap<String, serde_json::Value>;
// ANCHOR_END: row-type
// ANCHOR: app-state
#[derive(Debug, Clone)]
pub struct AppState {
    pub articles: BTreeMap<i64, Row>,
    pub authors: BTreeMap<i64, Row>,
    pub metrics: Metrics,
}
// ANCHOR_END: app-state
// ANCHOR: read_csv
fn read_csv(path: &str) -> Result<BTreeMap<i64, Row>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut records: BTreeMap<i64, Row> = BTreeMap::new();
    for row in rdr.deserialize() {
        let row: BTreeMap<String, serde_json::Value> = row?;
        let id = row
            .get("id")
            .ok_or("'id' field not found in csv file")?
            .as_i64()
            .ok_or("'id' field was not an integer in csv file")?;
        records.insert(id, row);
    }
    Ok(records)
}
// ANCHOR_END: read_csv

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
    // Read the CSV data files
    let articles = read_csv("articles.csv").unwrap();
    let authors = read_csv("authors.csv").unwrap();

    let metrics = Metrics::new().unwrap();

    AppState {
        articles,
        authors,
        metrics,
    }
}
// ANCHOR_END: init_app_state

type StatusLine = (StatusCode, &'static str);
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
async fn get_metrics(State(state): State<Arc<Mutex<AppState>>>) -> Result<String, StatusLine> {
    let state = state.lock().await;
    state
        .metrics
        .as_text()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "cannot encode metrics"))
}
// ANCHOR_END: metrics
// ANCHOR: capabilities
async fn get_capabilities() -> Json<models::CapabilitiesResponse> {
    let empty = serde_json::to_value(BTreeMap::<String, ()>::new()).unwrap();
    Json(models::CapabilitiesResponse {
        versions: "^0.1.0".into(),
        capabilities: models::Capabilities {
            explain: None,
            query: Some(models::QueryCapabilities {
                foreach: Some(empty.clone()),
                order_by_aggregate: Some(empty.clone()),
                relation_comparisons: Some(empty.clone()),
            }),
            mutations: Some(models::MutationCapabilities {
                returning: Some(empty.clone()),
                nested_inserts: Some(empty.clone()),
            }),
            relationships: Some(empty),
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
                update_operators: BTreeMap::new(),
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
                update_operators: BTreeMap::new(),
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
                    arguments: BTreeMap::new(),
                    r#type: models::Type::Named { name: "Int".into() },
                },
            ),
            (
                "title".into(),
                models::ObjectField {
                    description: Some("The article's title".into()),
                    arguments: BTreeMap::new(),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "author_id".into(),
                models::ObjectField {
                    description: Some("The article's author ID".into()),
                    arguments: BTreeMap::new(),
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
                    arguments: BTreeMap::new(),
                    r#type: models::Type::Named { name: "Int".into() },
                },
            ),
            (
                "first_name".into(),
                models::ObjectField {
                    description: Some("The author's first name".into()),
                    arguments: BTreeMap::new(),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "last_name".into(),
                models::ObjectField {
                    description: Some("The author's last name".into()),
                    arguments: BTreeMap::new(),
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
        deletable: false,
        insertable_columns: None,
        updatable_columns: None,
        foreign_keys: BTreeMap::new(),
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
        deletable: false,
        insertable_columns: None,
        updatable_columns: None,
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
        deletable: false,
        insertable_columns: None,
        updatable_columns: None,
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
) -> Result<Json<models::QueryResponse>, StatusLine> {
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
    collection: &String,
    arguments: &BTreeMap<String, models::Argument>,
    collection_relationships: &BTreeMap<String, models::Relationship>,
    query: &models::Query,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
) -> Result<models::RowSet, StatusLine> {
    // ANCHOR_END: execute_query_with_variables_signature
    let mut argument_values = BTreeMap::new();

    for (argument_name, argument_value) in arguments.iter() {
        if let Some(_) = argument_values.insert(
            argument_name.clone(),
            eval_argument(variables, argument_value)?,
        ) {
            return Err((StatusCode::BAD_REQUEST, "duplicate argument names"));
        }
    }

    execute_query_by_collection_name(
        &collection_relationships,
        variables,
        collection.as_str(),
        &argument_values,
        None,
        &query,
        state,
    )
}
// ANCHOR_END: execute_query_with_variables
// ANCHOR: execute_query_by_collection_name
fn execute_query_by_collection_name(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    collection_name: &str,
    arguments: &BTreeMap<String, serde_json::Value>,
    root: Option<&Row>,
    query: &models::Query,
    state: &AppState,
) -> Result<models::RowSet, StatusLine> {
    let collection = get_collection_by_name(collection_name, arguments, state)?;
    execute_query(
        collection_relationships,
        variables,
        state,
        query,
        root,
        collection,
    )
}
// ANCHOR_END: execute_query_by_collection_name
// ANCHOR: get_collection_by_name
fn get_collection_by_name(
    collection_name: &str,
    arguments: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
) -> Result<Vec<Row>, StatusLine> {
    match collection_name {
        "articles" => Ok(state.articles.values().cloned().collect()),
        "authors" => Ok(state.authors.values().cloned().collect()),
        "articles_by_author" => {
            let author_id = arguments
                .get("author_id".into())
                .ok_or((StatusCode::BAD_REQUEST, "missing argument author_id"))?;
            let author_id_int = author_id
                .as_i64()
                .ok_or((StatusCode::BAD_REQUEST, "author_id must be a string"))?;

            let mut articles_by_author = vec![];

            for (_id, article) in state.articles.iter() {
                let article_author_id = article
                    .get("author_id")
                    .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "author_id not found"))?;
                let article_author_id_int = article_author_id.as_i64().ok_or((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "author_id must be a string",
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
                    "cannot encode article id",
                )
            })?;
            Ok(vec![BTreeMap::from_iter([(
                "__value".into(),
                latest_id_value,
            )])])
        }
        _ => Err((StatusCode::BAD_REQUEST, "invalid collection name")),
    }
}
// ANCHOR_END: get_collection_by_name
// ANCHOR: execute_query
// ANCHOR: execute_query_signature
fn execute_query(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    query: &models::Query,
    root: Option<&Row>,
    collection: Vec<Row>,
) -> Result<models::RowSet, StatusLine> {
    // ANCHOR_END: execute_query_signature
    // ANCHOR: execute_query_sort
    let sorted = sort(
        collection_relationships,
        variables,
        state,
        collection,
        root,
        &query.order_by,
    )?;
    // ANCHOR_END: execute_query_sort
    // ANCHOR: execute_query_filter
    let filtered: Vec<Row> = (match &query.predicate {
        None => Ok::<_, StatusLine>(sorted),
        Some(expr) => {
            let mut filtered: Vec<Row> = vec![];
            for item in sorted.into_iter() {
                let root = root.unwrap_or(&item);
                if eval_expression(
                    collection_relationships,
                    variables,
                    state,
                    &expr,
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
            let mut rows: Vec<models::Row> = vec![];
            for item in paginated.iter() {
                let mut row = models::Row {
                    columns: IndexMap::new(),
                    relationships: IndexMap::new(),
                };
                let root = root.unwrap_or(item);
                for (field_name, field) in fields.iter() {
                    eval_field(
                        collection_relationships,
                        variables,
                        state,
                        field,
                        root,
                        &field_name,
                        &mut row,
                        item,
                    )?;
                }
                rows.push(row)
            }
            Ok::<_, StatusLine>(rows)
        })
        .transpose()?;
    // ANCHOR_END: execute_query_fields
    // ANCHOR: execute_query_rowset
    Ok(models::RowSet { aggregates, rows })
    // ANCHOR_END: execute_query_rowset
}
// ANCHOR_END: execute_query
// ANCHOR: eval_aggregate
fn eval_aggregate(
    aggregate: &models::Aggregate,
    paginated: &Vec<BTreeMap<String, serde_json::Value>>,
) -> Result<serde_json::Value, StatusLine> {
    match aggregate {
        models::Aggregate::StarCount {} => Ok(serde_json::Value::from(paginated.len())),
        models::Aggregate::ColumnCount { column, distinct } => {
            let values = paginated
                .iter()
                .map(|row| {
                    row.get(column)
                        .ok_or((StatusCode::BAD_REQUEST, "invalid column name"))
                })
                .collect::<Result<Vec<_>, StatusLine>>()?;

            let non_null_values = values.iter().filter(|value| !value.is_null());

            let agg_value = if *distinct {
                non_null_values
                    .map(|value| {
                        serde_json::to_string(value).map_err(|_| {
                            (StatusCode::INTERNAL_SERVER_ERROR, "unable to encode value")
                        })
                    })
                    .collect::<HashSet<_>>()
                    .len()
            } else {
                non_null_values.count()
            };
            serde_json::to_value(agg_value).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "unable to encode response",
                )
            })
        }
        models::Aggregate::SingleColumn { column, function } => {
            let values = paginated
                .iter()
                .map(|row| {
                    row.get(column)
                        .ok_or((StatusCode::BAD_REQUEST, "invalid column name"))
                })
                .collect::<Result<Vec<_>, StatusLine>>()?;
            eval_aggregate_function(function, values)
        }
    }
}
// ANCHOR_END: eval_aggregate
// ANCHOR: eval_aggregate_function
fn eval_aggregate_function(
    function: &String,
    values: Vec<&serde_json::Value>,
) -> Result<serde_json::Value, StatusLine> {
    let int_values = values
        .iter()
        .map(|value| {
            value
                .as_i64()
                .ok_or((StatusCode::BAD_REQUEST, "column is not an integer"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let agg_value = match function.as_str() {
        "min" => Ok(int_values.iter().min()),
        "max" => Ok(int_values.iter().max()),
        _ => Err((StatusCode::BAD_REQUEST, "invalid aggregation function")),
    }?;
    serde_json::to_value(agg_value).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "unable to encode response",
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
    root: Option<&Row>,
    order_by: &Option<models::OrderBy>,
) -> Result<Vec<Row>, StatusLine> {
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
                        root,
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
    root: Option<&Row>,
    t1: &Row,
    t2: &Row,
) -> Result<Ordering, StatusLine> {
    let mut result = Ordering::Equal;

    for element in order_by.elements.iter() {
        let v1 = eval_order_by_element(
            collection_relationships,
            variables,
            state,
            element,
            root.unwrap_or(t1),
            t1,
        )?;
        let v2 = eval_order_by_element(
            collection_relationships,
            variables,
            state,
            element,
            root.unwrap_or(t2),
            t2,
        )?;
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
fn compare(v1: serde_json::Value, v2: serde_json::Value) -> Result<Ordering, StatusLine> {
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
        _ => Err((StatusCode::INTERNAL_SERVER_ERROR, "cannot compare values")),
    }
}
// ANCHOR_END: compare
// ANCHOR: eval_order_by_element
fn eval_order_by_element(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    element: &models::OrderByElement,
    root: &Row,
    item: &Row,
) -> Result<serde_json::Value, StatusLine> {
    match element.target.clone() {
        models::OrderByTarget::Column { name, path } => eval_order_by_column(
            collection_relationships,
            variables,
            state,
            root,
            item,
            path,
            name,
        ),
        models::OrderByTarget::SingleColumnAggregate {
            column,
            function,
            path,
        } => eval_order_by_single_column_aggregate(
            collection_relationships,
            variables,
            state,
            root,
            item,
            path,
            column,
            function,
        ),
        models::OrderByTarget::StarCountAggregate { path } => eval_order_by_star_count_aggregate(
            collection_relationships,
            variables,
            state,
            root,
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
    root: &BTreeMap<String, serde_json::Value>,
    item: &BTreeMap<String, serde_json::Value>,
    path: Vec<models::PathElement>,
) -> Result<serde_json::Value, StatusLine> {
    let rows: Vec<Row> = eval_path(
        collection_relationships,
        variables,
        state,
        &path,
        root,
        item,
    )?;
    Ok(rows.len().into())
}
// ANCHOR_END: eval_order_by_star_count_aggregate
// ANCHOR: eval_order_by_single_column_aggregate
fn eval_order_by_single_column_aggregate(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    root: &BTreeMap<String, serde_json::Value>,
    item: &BTreeMap<String, serde_json::Value>,
    path: Vec<models::PathElement>,
    column: String,
    function: String,
) -> Result<serde_json::Value, StatusLine> {
    let rows: Vec<Row> = eval_path(
        collection_relationships,
        variables,
        state,
        &path,
        root,
        item,
    )?;
    let values = rows
        .iter()
        .map(|row| {
            row.get(column.as_str())
                .ok_or((StatusCode::BAD_REQUEST, "invalid column name"))
        })
        .collect::<Result<Vec<_>, StatusLine>>()?;
    eval_aggregate_function(&function, values)
}
// ANCHOR_END: eval_order_by_single_column_aggregate
// ANCHOR: eval_order_by_column
fn eval_order_by_column(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    root: &BTreeMap<String, serde_json::Value>,
    item: &BTreeMap<String, serde_json::Value>,
    path: Vec<models::PathElement>,
    name: String,
) -> Result<serde_json::Value, StatusLine> {
    let rows: Vec<Row> = eval_path(
        collection_relationships,
        variables,
        state,
        &path,
        root,
        item,
    )?;
    if rows.len() > 1 {
        return Err((
            StatusCode::BAD_REQUEST,
            "cannot order by column via array relationship",
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
    path: &Vec<models::PathElement>,
    root: &Row,
    item: &Row,
) -> Result<Vec<Row>, StatusLine> {
    let mut result: Vec<Row> = vec![item.clone()];

    for path_element in path.iter() {
        let relationship_name = path_element.relationship.as_str();
        let relationship = collection_relationships
            .get(relationship_name)
            .ok_or((StatusCode::BAD_REQUEST, "invalid relationship name in path"))?;
        result = eval_path_element(
            collection_relationships,
            variables,
            state,
            relationship,
            &path_element.arguments,
            root,
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
    root: &Row,
    source: &Vec<Row>,
    predicate: &models::Expression,
) -> Result<Vec<Row>, StatusLine> {
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
            if let Some(_) = all_arguments.insert(
                argument_name.clone(),
                eval_relationship_argument(variables, src_row, argument_value)?,
            ) {
                return Err((StatusCode::BAD_REQUEST, "duplicate argument names"));
            }
        }

        for (argument_name, argument_value) in arguments.iter() {
            if let Some(_) = all_arguments.insert(
                argument_name.clone(),
                eval_relationship_argument(variables, src_row, argument_value)?,
            ) {
                return Err((StatusCode::BAD_REQUEST, "duplicate argument names"));
            }
        }

        let target = get_collection_by_name(
            relationship.target_collection.as_str(),
            &all_arguments,
            state,
        )?;

        for tgt_row in target.iter() {
            if eval_column_mapping(relationship, src_row, tgt_row)? {
                if eval_expression(
                    collection_relationships,
                    variables,
                    state,
                    &predicate,
                    root,
                    &tgt_row,
                )? {
                    matching_rows.push(tgt_row.clone());
                }
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
) -> Result<serde_json::Value, StatusLine> {
    match argument {
        models::Argument::Variable { name } => {
            let value = variables
                .get(name.as_str())
                .ok_or((StatusCode::BAD_REQUEST, "invalid variable name"))
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
) -> Result<serde_json::Value, StatusLine> {
    match argument {
        models::RelationshipArgument::Variable { name } => {
            let value = variables
                .get(name.as_str())
                .ok_or((StatusCode::BAD_REQUEST, "invalid variable name"))
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
) -> Result<bool, StatusLine> {
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
        models::Expression::UnaryComparisonOperator { column, operator } => match &**operator {
            models::UnaryComparisonOperator::IsNull => {
                let vals = eval_comparison_target(
                    collection_relationships,
                    variables,
                    state,
                    &*column,
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
        } => match &**operator {
            models::BinaryComparisonOperator::Equal => {
                let left_vals = eval_comparison_target(
                    collection_relationships,
                    variables,
                    state,
                    &*column,
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
                        &*column,
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
                            let column_str = column_val
                                .as_str()
                                .ok_or((StatusCode::BAD_REQUEST, "column is not a string"))?;
                            let regex_str = regex_val.as_str().ok_or((
                                StatusCode::BAD_REQUEST,
                                "regular expression is not a string",
                            ))?;
                            let regex = Regex::new(regex_str.into()).map_err(|_| {
                                (StatusCode::BAD_REQUEST, "invalid regular expression")
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
                    "invalid binary comparison operator",
                )),
            },
            // ANCHOR_END: eval_expression_custom_binary_operators
        },
        // ANCHOR: eval_expression_binary_array_operators
        models::Expression::BinaryArrayComparisonOperator {
            column,
            operator,
            values,
        } => match &**operator {
            models::BinaryArrayComparisonOperator::In => {
                let left_val = eval_comparison_target(
                    collection_relationships,
                    variables,
                    state,
                    &*column,
                    root,
                    item,
                )?;

                for v in values.iter() {
                    let right_val = eval_comparison_value(
                        collection_relationships,
                        variables,
                        state,
                        v,
                        root,
                        item,
                    )?;
                    if left_val == right_val {
                        return Ok(true);
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
            let row_set = eval_in_collection(
                collection_relationships,
                item,
                variables,
                state,
                root,
                query,
                in_collection,
            )?;
            let rows: Vec<models::Row> = row_set.rows.ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                "exists query returned no rows",
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
    root: &BTreeMap<String, serde_json::Value>,
    query: models::Query,
    in_collection: &Box<models::ExistsInCollection>,
) -> Result<models::RowSet, StatusLine> {
    let row_set = match &**in_collection {
        models::ExistsInCollection::Related {
            relationship,
            arguments,
        } => {
            let relationship = collection_relationships.get(relationship.as_str()).ok_or((
                StatusCode::BAD_REQUEST,
                "invalid relationship name in exists predicate",
            ))?;
            let source = vec![item.clone()];
            let collection = eval_path_element(
                collection_relationships,
                variables,
                state,
                relationship,
                arguments,
                root,
                &source,
                &models::Expression::And {
                    expressions: vec![],
                },
            )?;
            execute_query(
                collection_relationships,
                variables,
                state,
                &query,
                Some(root),
                collection,
            )
        }
        models::ExistsInCollection::Unrelated {
            collection,
            arguments,
        } => {
            let arguments = arguments
                .iter()
                .map(|(k, v)| Ok((k.clone(), eval_relationship_argument(variables, item, v)?)))
                .collect::<Result<BTreeMap<_, _>, _>>()?;
            execute_query_by_collection_name(
                collection_relationships,
                variables,
                collection.as_str(),
                &arguments,
                Some(root),
                &query,
                state,
            )
        }
    }?;
    Ok(row_set)
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
) -> Result<Vec<serde_json::Value>, StatusLine> {
    match target {
        models::ComparisonTarget::Column { name, path } => {
            let rows = eval_path(collection_relationships, variables, state, path, root, item)?;
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
fn eval_column(row: &Row, column_name: &str) -> Result<serde_json::Value, StatusLine> {
    row.get(column_name)
        .cloned()
        .ok_or((StatusCode::BAD_REQUEST, "invalid column name"))
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
) -> Result<Vec<serde_json::Value>, StatusLine> {
    match comparison_value {
        models::ComparisonValue::Column { column } => eval_comparison_target(
            collection_relationships,
            variables,
            state,
            &*column,
            root,
            item,
        ),
        models::ComparisonValue::Scalar { value } => Ok(vec![value.clone()]),
        models::ComparisonValue::Variable { name } => {
            let value = variables
                .get(name.as_str())
                .ok_or((StatusCode::BAD_REQUEST, "invalid variable name"))
                .cloned()?;
            Ok(vec![value])
        }
    }
}
// ANCHOR_END: eval_comparison_value
// ANCHOR: eval_field
fn eval_field(
    collection_relationships: &BTreeMap<String, models::Relationship>,
    variables: &BTreeMap<String, serde_json::Value>,
    state: &AppState,
    field: &models::Field,
    root: &Row,
    field_name: &String,
    row: &mut models::Row,
    item: &Row,
) -> Result<(), StatusLine> {
    match field {
        models::Field::Column { column, .. } => {
            row.columns
                .insert(field_name.clone(), eval_column(item, column.as_str())?);
            Ok(())
        }
        models::Field::Relationship {
            relationship,
            arguments,
            query,
        } => {
            let relationship = collection_relationships.get(relationship.as_str()).ok_or((
                StatusCode::BAD_REQUEST,
                "invalid relationship name in field",
            ))?;
            let source = vec![item.clone()];
            let collection = eval_path_element(
                collection_relationships,
                variables,
                state,
                relationship,
                arguments,
                root,
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
                Some(root),
                collection,
            )?;
            row.relationships.insert(field_name.clone(), rows);
            Ok(())
        }
    }
}
// ANCHOR_END: eval_field
// ANCHOR: explain
async fn post_explain(
    Json(_request): Json<models::QueryRequest>,
) -> Result<Json<models::ExplainResponse>, StatusLine> {
    Err((StatusCode::NOT_IMPLEMENTED, "explain is not supported"))
}
// ANCHOR_END: explain
// ANCHOR: mutation
async fn post_mutation(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(request): Json<models::MutationRequest>,
) -> Result<Json<models::MutationResponse>, StatusLine> {
    let mut state = state.lock().await;

    let mut operation_results = vec![];

    for operation in request.operations.iter() {
        let operation_result = execute_mutation_operation(
            &mut state,
            &request.insert_schema,
            &request.collection_relationships,
            operation,
        )
        .await?;
        operation_results.push(operation_result);
    }

    Ok(Json(models::MutationResponse { operation_results }))
}
// ANCHOR_END: mutation

async fn execute_mutation_operation(
    state: &mut AppState,
    _insert_schema: &Vec<models::CollectionInsertSchema>,
    collection_relationships: &BTreeMap<String, models::Relationship>,
    operation: &models::MutationOperation,
) -> Result<models::MutationOperationResults, StatusLine> {
    match operation {
        models::MutationOperation::Insert {
            post_insert_check: _,
            returning_fields: _,
            rows: _,
            collection: _,
        } => {
            todo!()
        }
        models::MutationOperation::Delete {
            returning_fields: _,
            collection: _,
            predicate: _,
        } => {
            todo!()
        }
        models::MutationOperation::Update {
            post_update_check: _,
            returning_fields: _,
            collection: _,
            updates: _,
            r#where: _,
        } => {
            todo!()
        }
        models::MutationOperation::Procedure {
            name,
            arguments,
            fields,
        } => match name.as_str() {
            "upsert_article" => {
                let article = arguments.get("article").ok_or((
                    StatusCode::BAD_REQUEST,
                    "argument 'article' is required in upsert_article",
                ))?;
                let article_obj = article.as_object().ok_or((
                    StatusCode::BAD_REQUEST,
                    "argument 'article' is not an object",
                ))?;
                let id = article_obj.get("id").ok_or((
                    StatusCode::BAD_REQUEST,
                    "argument 'article' is missing required field 'id'",
                ))?;
                let id_int = id.as_i64().ok_or((
                    StatusCode::BAD_REQUEST,
                    "argument 'article.id' is not an integer",
                ))?;
                let new_row =
                    BTreeMap::from_iter(article_obj.iter().map(|(k, v)| (k.clone(), v.clone())));
                let old_row = state.articles.insert(id_int, new_row);
                let returning = old_row
                    .map(|old_row| {
                        let mut row = models::Row {
                            columns: IndexMap::new(),
                            relationships: IndexMap::new(),
                        };
                        for fields in fields.iter() {
                            for (field_name, field) in fields.iter() {
                                eval_field(
                                    collection_relationships,
                                    &BTreeMap::new(),
                                    state,
                                    field,
                                    &old_row,
                                    &field_name,
                                    &mut row,
                                    &old_row,
                                )?;
                            }
                        }
                        Ok(vec![row])
                    })
                    .transpose()?;
                Ok(models::MutationOperationResults {
                    affected_rows: 1,
                    returning,
                })
            }
            _ => Err((StatusCode::BAD_REQUEST, "unknown procedure")),
        },
    }
}

fn eval_column_mapping(
    relationship: &models::Relationship,
    src_row: &Row,
    tgt_row: &Row,
) -> Result<bool, StatusLine> {
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
    use axum::{extract::State, Json};
    use goldenfile::Mint;
    use ndc_client::models;
    use std::{
        fs::{self, File},
        io::Write,
        path::PathBuf,
        sync::Arc,
    };
    use tokio::sync::Mutex;

    #[test]
    fn test_capabilities() {
        tokio_test::block_on(async {
            let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

            let mut mint = Mint::new(&test_dir);

            let expected_path = PathBuf::from_iter(["capabilities", "expected.json"]);

            let response = crate::get_capabilities().await;

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

                let response_json = serde_json::to_string_pretty(&response.0).unwrap();

                write!(expected, "{}", response_json).unwrap();

                // Test roundtrip
                assert_eq!(
                    response.0,
                    serde_json::from_str(response_json.as_str()).unwrap()
                );
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
}
