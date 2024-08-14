use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashSet},
    env,
    error::Error,
    net,
    sync::Arc,
};

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use indexmap::IndexMap;
use itertools::Itertools;
use ndc_models::{self as models};
use prometheus::{Encoder, IntCounter, IntGauge, Opts, Registry, TextEncoder};
use regex::Regex;
use tokio::sync::Mutex;

const DEFAULT_PORT: u16 = 8080;

const ARTICLES_JSON: &str = include_str!("../../articles.json");
const AUTHORS_JSON: &str = include_str!("../../authors.json");
const INSTITUTIONS_JSON: &str = include_str!("../../institutions.json");

// ANCHOR: row-type
type Row = BTreeMap<models::FieldName, serde_json::Value>;
// ANCHOR_END: row-type
// ANCHOR: app-state
#[derive(Debug, Clone)]
pub struct AppState {
    pub articles: BTreeMap<i32, Row>,
    pub authors: BTreeMap<i32, Row>,
    pub institutions: BTreeMap<i32, Row>,
    pub metrics: Metrics,
}
// ANCHOR_END: app-state

// ANCHOR: read_json_lines
fn read_json_lines(contents: &str) -> core::result::Result<BTreeMap<i32, Row>, Box<dyn Error>> {
    let mut records: BTreeMap<i32, Row> = BTreeMap::new();
    for line in contents.lines() {
        let row: BTreeMap<models::FieldName, serde_json::Value> = serde_json::from_str(line)?;
        let id: i32 = row
            .get("id")
            .ok_or("'id' field not found in json file")?
            .as_i64()
            .ok_or("'id' field was not an integer in json file")?
            .try_into()?;
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
async fn metrics_middleware(
    state: State<Arc<Mutex<AppState>>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
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
    let articles = read_json_lines(ARTICLES_JSON).unwrap();
    let authors = read_json_lines(AUTHORS_JSON).unwrap();
    let institutions = read_json_lines(INSTITUTIONS_JSON).unwrap();

    let metrics = Metrics::new().unwrap();

    AppState {
        articles,
        authors,
        institutions,
        metrics,
    }
}
// ANCHOR_END: init_app_state

type Result<A> = core::result::Result<A, (StatusCode, Json<models::ErrorResponse>)>;

// ANCHOR: main
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    let app_state = Arc::new(Mutex::new(init_app_state()));

    let app = Router::new()
        .route("/health", get(get_health))
        .route("/metrics", get(get_metrics))
        .route("/capabilities", get(get_capabilities))
        .route("/schema", get(get_schema))
        .route("/query", post(post_query))
        .route("/query/explain", post(post_query_explain))
        .route("/mutation", post(post_mutation))
        .route("/mutation/explain", post(post_mutation_explain))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&app_state),
            metrics_middleware,
        ))
        .with_state(app_state);

    // Start the server on `localhost:<PORT>`.
    // This says it's binding to an IPv6 address, but will actually listen to
    // any IPv4 or IPv6 address.
    let host = net::IpAddr::V6(net::Ipv6Addr::UNSPECIFIED);
    let port = env::var("PORT")
        .map(|s| s.parse())
        .unwrap_or(Ok(DEFAULT_PORT))?;
    let addr = net::SocketAddr::new(host, port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Serving on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_handler())
        .await?;

    Ok(())
}
// ANCHOR_END: main
async fn shutdown_handler() {
    // Wait for a SIGINT, i.e. a Ctrl+C from the keyboard
    let sigint = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install signal handler");
    };
    // Wait for a SIGTERM, i.e. a normal `kill` command
    #[cfg(unix)]
    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await
    };
    // Block until either of the above happens
    #[cfg(unix)]
    tokio::select! {
        () = sigint => (),
        _ = sigterm => (),
    }
    #[cfg(windows)]
    tokio::select! {
        _ = sigint => (),
    }
}
// ANCHOR: health
async fn get_health() -> StatusCode {
    StatusCode::OK
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
        version: models::VERSION.into(),
        capabilities: models::Capabilities {
            query: models::QueryCapabilities {
                aggregates: Some(models::AggregateCapabilities {
                    filter_by: Some(models::LeafCapability {}),
                    group_by: Some(models::GroupByCapabilities {
                        filter: Some(models::LeafCapability {}),
                        order: Some(models::LeafCapability {}),
                        paginate: Some(models::LeafCapability {}),
                    }),
                }),
                variables: Some(models::LeafCapability {}),
                exists: models::ExistsCapabilities {
                    named_scopes: Some(models::LeafCapability {}),
                    unrelated: Some(models::LeafCapability {}),
                    nested_collections: Some(models::LeafCapability {}),
                },
                explain: None,
                nested_fields: models::NestedFieldCapabilities {
                    filter_by: Some(models::LeafCapability {}),
                    order_by: Some(models::LeafCapability {}),
                    aggregates: Some(models::LeafCapability {}),
                    nested_collections: Some(models::LeafCapability {}),
                },
            },
            mutation: models::MutationCapabilities {
                transactional: None,
                explain: None,
            },
            relationships: Some(models::RelationshipCapabilities {
                order_by_aggregate: Some(models::LeafCapability {}),
                relation_comparisons: Some(models::LeafCapability {}),
            }),
        },
    })
}
// ANCHOR_END: capabilities
// ANCHOR: schema1
async fn get_schema() -> Json<models::SchemaResponse> {
    // ANCHOR_END: schema1
    let array_arguments: BTreeMap<models::ArgumentName, models::ArgumentInfo> = vec![(
        models::ArgumentName::from("limit"),
        models::ArgumentInfo {
            description: None,
            argument_type: models::Type::Nullable {
                underlying_type: Box::new(models::Type::Named { name: "Int".into() }),
            },
        },
    )]
    .into_iter()
    .collect();
    // ANCHOR: schema_scalar_types
    let scalar_types = BTreeMap::from_iter([
        (
            "String".into(),
            models::ScalarType {
                representation: Some(models::TypeRepresentation::String),
                aggregate_functions: BTreeMap::new(),
                comparison_operators: BTreeMap::from_iter([
                    ("eq".into(), models::ComparisonOperatorDefinition::Equal),
                    ("in".into(), models::ComparisonOperatorDefinition::In),
                    (
                        "like".into(),
                        models::ComparisonOperatorDefinition::Custom {
                            argument_type: models::Type::Named {
                                name: "String".into(),
                            },
                        },
                    ),
                ]),
            },
        ),
        (
            "Int".into(),
            models::ScalarType {
                representation: Some(models::TypeRepresentation::Int32),
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
                comparison_operators: BTreeMap::from_iter([
                    ("eq".into(), models::ComparisonOperatorDefinition::Equal),
                    ("in".into(), models::ComparisonOperatorDefinition::In),
                ]),
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
                    arguments: BTreeMap::new(),
                },
            ),
            (
                "title".into(),
                models::ObjectField {
                    description: Some("The article's title".into()),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                    arguments: BTreeMap::new(),
                },
            ),
            (
                "author_id".into(),
                models::ObjectField {
                    description: Some("The article's author ID".into()),
                    r#type: models::Type::Named { name: "Int".into() },
                    arguments: BTreeMap::new(),
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
                    arguments: BTreeMap::new(),
                },
            ),
            (
                "first_name".into(),
                models::ObjectField {
                    description: Some("The author's first name".into()),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                    arguments: BTreeMap::new(),
                },
            ),
            (
                "last_name".into(),
                models::ObjectField {
                    description: Some("The author's last name".into()),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                    arguments: BTreeMap::new(),
                },
            ),
        ]),
    };
    // ANCHOR_END: schema_object_type_author
    // ANCHOR: schema_object_type_institution
    let institution_type = models::ObjectType {
        description: Some("An institution".into()),
        fields: BTreeMap::from_iter([
            (
                "id".into(),
                models::ObjectField {
                    description: Some("The institution's primary key".into()),
                    r#type: models::Type::Named { name: "Int".into() },
                    arguments: BTreeMap::new(),
                },
            ),
            (
                "name".into(),
                models::ObjectField {
                    description: Some("The institution's name".into()),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                    arguments: BTreeMap::new(),
                },
            ),
            (
                "location".into(),
                models::ObjectField {
                    description: Some("The institution's location".into()),
                    r#type: models::Type::Named {
                        name: "location".into(),
                    },
                    arguments: BTreeMap::new(),
                },
            ),
            (
                "staff".into(),
                models::ObjectField {
                    description: Some("The institution's staff".into()),
                    r#type: models::Type::Array {
                        element_type: Box::new(models::Type::Named {
                            name: "staff_member".into(),
                        }),
                    },
                    arguments: array_arguments.clone(),
                },
            ),
            (
                "departments".into(),
                models::ObjectField {
                    description: Some("The institution's departments".into()),
                    r#type: models::Type::Array {
                        element_type: Box::new(models::Type::Named {
                            name: "String".into(),
                        }),
                    },
                    arguments: array_arguments.clone(),
                },
            ),
        ]),
    };
    // ANCHOR_END: schema_object_type_institution
    // ANCHOR: schema_object_type_location
    let location_type = models::ObjectType {
        description: Some("A location".into()),
        fields: BTreeMap::from_iter([
            (
                "city".into(),
                models::ObjectField {
                    description: Some("The location's city".into()),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                    arguments: BTreeMap::new(),
                },
            ),
            (
                "country".into(),
                models::ObjectField {
                    description: Some("The location's country".into()),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                    arguments: BTreeMap::new(),
                },
            ),
            (
                "campuses".into(),
                models::ObjectField {
                    description: Some("The location's campuses".into()),
                    r#type: models::Type::Array {
                        element_type: Box::new(models::Type::Named {
                            name: "String".into(),
                        }),
                    },
                    arguments: array_arguments.clone(),
                },
            ),
        ]),
    };
    // ANCHOR_END: schema_object_type_location
    // ANCHOR: schema_object_type_staff_member
    let staff_member_type = models::ObjectType {
        description: Some("A staff member".into()),
        fields: BTreeMap::from_iter([
            (
                "first_name".into(),
                models::ObjectField {
                    description: Some("The staff member's first name".into()),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                    arguments: BTreeMap::new(),
                },
            ),
            (
                "last_name".into(),
                models::ObjectField {
                    description: Some("The staff member's last name".into()),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                    arguments: BTreeMap::new(),
                },
            ),
            (
                "specialities".into(),
                models::ObjectField {
                    description: Some("The staff member's specialities".into()),
                    r#type: models::Type::Array {
                        element_type: Box::new(models::Type::Named {
                            name: "String".into(),
                        }),
                    },
                    arguments: array_arguments.clone(),
                },
            ),
        ]),
    };
    // ANCHOR_END: schema_object_type_staff_member
    // ANCHOR: schema_object_types
    let object_types = BTreeMap::from_iter([
        ("article".into(), article_type),
        ("author".into(), author_type),
        ("institution".into(), institution_type),
        ("location".into(), location_type),
        ("staff_member".into(), staff_member_type),
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
    // ANCHOR: schema_collection_institution
    let institutions_collection = models::CollectionInfo {
        name: "institutions".into(),
        description: Some("A collection of institutions".into()),
        collection_type: "institution".into(),
        arguments: BTreeMap::new(),
        foreign_keys: BTreeMap::new(),
        uniqueness_constraints: BTreeMap::from_iter([(
            "InstitutionByID".into(),
            models::UniquenessConstraint {
                unique_columns: vec!["id".into()],
            },
        )]),
    };
    // ANCHOR_END: schema_collection_institution
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
        institutions_collection,
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
    // ANCHOR: schema_procedure_delete_articles
    let delete_articles = models::ProcedureInfo {
        name: "delete_articles".into(),
        description: Some("Delete articles which match a predicate".into()),
        arguments: BTreeMap::from_iter([(
            "where".into(),
            models::ArgumentInfo {
                description: Some("The predicate".into()),
                argument_type: models::Type::Predicate {
                    object_type_name: "article".into(),
                },
            },
        )]),
        result_type: models::Type::Array {
            element_type: Box::new(models::Type::Named {
                name: "article".into(),
            }),
        },
    };
    // ANCHOR_END: schema_procedure_delete_article
    // ANCHOR: schema_procedures
    let procedures = vec![upsert_article, delete_articles];
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
    // ANCHOR: schema_function_latest_article
    let latest_article_function = models::FunctionInfo {
        name: "latest_article".into(),
        description: Some("Get the most recent article".into()),
        result_type: models::Type::Nullable {
            underlying_type: Box::new(models::Type::Named {
                name: "article".into(),
            }),
        },
        arguments: BTreeMap::new(),
    };
    // ANCHOR_END: schema_function_latest_article
    // ANCHOR: schema_functions
    let functions: Vec<models::FunctionInfo> =
        vec![latest_article_id_function, latest_article_function];
    // ANCHOR_END: schema_functions
    // ANCHOR: schema_capabilities
    let capabilities = Some(models::CapabilitySchemaInfo {
        query: Some(models::QueryCapabilitiesSchemaInfo {
            aggregates: Some(ndc_models::AggregateCapabilitiesSchemaInfo {
                filter_by: Some(ndc_models::AggregateFilterByCapabilitiesSchemaInfo {
                    count_scalar_type: "Int".into(),
                }),
            }),
        }),
    });
    // ANCHOR_END: schema_capabilities
    // ANCHOR: schema2
    Json(models::SchemaResponse {
        scalar_types,
        object_types,
        collections,
        functions,
        procedures,
        capabilities,
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

    for variables in &variable_sets {
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
    collection: &models::CollectionName,
    arguments: &BTreeMap<models::ArgumentName, models::Argument>,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    query: &models::Query,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
) -> Result<models::RowSet> {
    // ANCHOR_END: execute_query_with_variables_signature
    let mut argument_values = BTreeMap::new();

    for (argument_name, argument_value) in arguments {
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
        Root::Reset,
        collection,
    )
}
// ANCHOR_END: execute_query_with_variables
// ANCHOR: get_collection_by_name
fn get_collection_by_name(
    collection_name: &models::CollectionName,
    arguments: &BTreeMap<models::ArgumentName, serde_json::Value>,
    state: &AppState,
) -> Result<Vec<Row>> {
    match collection_name.as_str() {
        "articles" => Ok(state.articles.values().cloned().collect()),
        "authors" => Ok(state.authors.values().cloned().collect()),
        "institutions" => Ok(state.institutions.values().cloned().collect()),
        "articles_by_author" => {
            let author_id = arguments.get("author_id").ok_or((
                StatusCode::BAD_REQUEST,
                Json(models::ErrorResponse {
                    message: "missing argument author_id".into(),
                    details: serde_json::Value::Null,
                }),
            ))?;
            let author_id_int: i32 = author_id
                .as_i64()
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "author_id must be an integer".into(),
                        details: serde_json::Value::Null,
                    }),
                ))?
                .try_into()
                .map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(models::ErrorResponse {
                            message: "author_id out of range".into(),
                            details: serde_json::Value::Null,
                        }),
                    )
                })?;

            let mut articles_by_author = vec![];

            for article in state.articles.values() {
                let article_author_id = article.get("author_id").ok_or((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(models::ErrorResponse {
                        message: "author_id not found".into(),
                        details: serde_json::Value::Null,
                    }),
                ))?;
                let article_author_id_int: i32 = article_author_id
                    .as_i64()
                    .ok_or((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(models::ErrorResponse {
                            message: "author_id must be an integer".into(),
                            details: serde_json::Value::Null,
                        }),
                    ))?
                    .try_into()
                    .map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(models::ErrorResponse {
                                message: "author_id out of range".into(),
                                details: serde_json::Value::Null,
                            }),
                        )
                    })?;
                if article_author_id_int == author_id_int {
                    articles_by_author.push(article.clone());
                }
            }

            Ok(articles_by_author)
        }
        "latest_article_id" => {
            let latest_id = state.articles.keys().max();
            let latest_id_value = serde_json::to_value(latest_id).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(models::ErrorResponse {
                        message: "unable to encode value".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })?;
            Ok(vec![BTreeMap::from_iter([(
                "__value".into(),
                latest_id_value,
            )])])
        }
        "latest_article" => {
            let latest = state
                .articles
                .iter()
                .max_by_key(|(&id, _)| id)
                .map(|(_, article)| article);
            let latest_value = serde_json::to_value(latest).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(models::ErrorResponse {
                        message: "unable to encode value".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })?;
            Ok(vec![BTreeMap::from_iter([(
                "__value".into(),
                latest_value,
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
#[derive(Clone, Copy)]
enum Root<'a> {
    PushCurrentRow(&'a [&'a Row]),
    Reset,
}
/// ANCHOR_END: Root
// ANCHOR: execute_query
// ANCHOR: execute_query_signature
fn execute_query(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
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
            for item in sorted {
                let scopes: Vec<&Row> = match root {
                    Root::PushCurrentRow(scopes) => {
                        let mut scopes = scopes.to_vec();
                        scopes.push(&item);
                        scopes
                    }
                    Root::Reset => vec![&item],
                };
                if eval_expression(
                    collection_relationships,
                    variables,
                    state,
                    expr,
                    &scopes,
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
        .map(|aggregates| eval_aggregates(aggregates, &paginated))
        .transpose()?;
    // ANCHOR_END: execute_query_aggregates
    // ANCHOR: execute_query_groups
    let groups = query
        .groups
        .as_ref()
        .map(|grouping| {
            eval_groups(
                collection_relationships,
                variables,
                state,
                grouping,
                &paginated,
            )
        })
        .transpose()?;
    // ANCHOR_END: execute_query_groups
    // ANCHOR: execute_query_fields
    let rows = query
        .fields
        .as_ref()
        .map(|fields| {
            let mut rows: Vec<IndexMap<models::FieldName, models::RowFieldValue>> = vec![];
            for item in &paginated {
                let row = eval_row(fields, collection_relationships, variables, state, item)?;
                rows.push(row);
            }
            Ok(rows)
        })
        .transpose()?;
    // ANCHOR_END: execute_query_fields
    // ANCHOR: execute_query_rowset
    Ok(models::RowSet {
        aggregates,
        rows,
        groups,
    })
    // ANCHOR_END: execute_query_rowset
}
// ANCHOR_END: execute_query
// ANCHOR: eval_groups
// ANCHOR: eval_groups_partition
fn eval_groups(
    collection_relationships: &BTreeMap<models::RelationshipName, ndc_models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    grouping: &ndc_models::Grouping,
    paginated: &[Row],
) -> Result<Vec<ndc_models::Group>> {
    let chunks: Vec<Chunk> = paginated
        .iter()
        .chunk_by(|row| {
            eval_dimensions(
                collection_relationships,
                variables,
                state,
                row,
                &grouping.dimensions,
            )
            .expect("cannot eval dimensions")
        })
        .into_iter()
        .map(|(dimensions, rows)| Chunk {
            dimensions,
            rows: rows.cloned().collect(),
        })
        .collect();
    // ANCHOR_END: eval_groups_partition
    // ANCHOR: eval_groups_sort
    let sorted = group_sort(
        collection_relationships,
        variables,
        state,
        chunks,
        &grouping.order_by,
    )?;
    // ANCHOR_END: eval_groups_sort
    // ANCHOR: eval_groups_filter
    let mut groups: Vec<models::Group> = vec![];

    for chunk in &sorted {
        let dimensions = chunk.dimensions.clone();

        let mut aggregates: IndexMap<String, serde_json::Value> = IndexMap::new();
        for (aggregate_name, aggregate) in &grouping.aggregates {
            aggregates.insert(
                aggregate_name.clone(),
                eval_aggregate(aggregate, &chunk.rows)?,
            );
        }
        if let Some(predicate) = &grouping.predicate {
            if eval_group_expression(variables, predicate, &chunk.rows)? {
                groups.push(models::Group {
                    dimensions: dimensions.clone(),
                    aggregates,
                });
            }
        } else {
            groups.push(models::Group {
                dimensions: dimensions.clone(),
                aggregates,
            });
        }
    }
    // ANCHOR_END: eval_groups_filter
    // ANCHOR: eval_groups_paginate
    let paginated: Vec<models::Group> =
        paginate(groups.into_iter(), grouping.limit, grouping.offset);

    Ok(paginated)
}
// ANCHOR_END: eval_groups_paginate
// ANCHOR_END: eval_groups
// ANCHOR: eval_group_expression
fn eval_group_expression(
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    expr: &models::GroupExpression,
    rows: &[Row],
) -> Result<bool> {
    match expr {
        models::GroupExpression::And { expressions } => {
            for expr in expressions {
                if !eval_group_expression(variables, expr, rows)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        models::GroupExpression::Or { expressions } => {
            for expr in expressions {
                if eval_group_expression(variables, expr, rows)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        models::GroupExpression::Not { expression } => {
            let b = eval_group_expression(variables, expression, rows)?;
            Ok(!b)
        }
        models::GroupExpression::BinaryComparisonOperator {
            target,
            operator,
            value,
        } => {
            let left_val = eval_group_comparison_target(target, rows)?;
            let right_vals = eval_aggregate_comparison_value(variables, value)?;
            eval_comparison_operator(operator, &left_val, right_vals)
        }
        ndc_models::GroupExpression::UnaryComparisonOperator { target, operator } => match operator
        {
            models::UnaryComparisonOperator::IsNull => {
                let val = eval_group_comparison_target(target, rows)?;
                Ok(val.is_null())
            }
        },
    }
}
// ANCHOR_END: eval_group_expression
// ANCHOR: eval_aggregate_comparison_value
fn eval_aggregate_comparison_value(
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    comparison_value: &models::GroupComparisonValue,
) -> Result<Vec<serde_json::Value>> {
    match comparison_value {
        models::GroupComparisonValue::Scalar { value } => Ok(vec![value.clone()]),
        models::GroupComparisonValue::Variable { name } => {
            let value = variables
                .get(name)
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
// ANCHOR_END: eval_aggregate_comparison_value
// ANCHOR: Chunk
struct Chunk {
    pub dimensions: Vec<serde_json::Value>,
    pub rows: Vec<Row>,
}
// ANCHOR_END: Chunk
// ANCHOR: group_sort
fn group_sort(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    groups: Vec<Chunk>,
    order_by: &Option<models::GroupOrderBy>,
) -> Result<Vec<Chunk>> {
    match order_by {
        None => Ok(groups),
        Some(order_by) => {
            let mut copy: Vec<Chunk> = vec![];
            for item_to_insert in groups {
                let mut index = 0;
                for other in &copy {
                    if let Ordering::Greater = eval_group_order_by(
                        collection_relationships,
                        variables,
                        state,
                        order_by,
                        other,
                        &item_to_insert,
                    )? {
                        break;
                    }
                    index += 1;
                }
                copy.insert(index, item_to_insert);
            }
            Ok(copy)
        }
    }
}
// ANCHOR_END: group_sort

// ANCHOR: eval_group_order_by
fn eval_group_order_by(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    order_by: &models::GroupOrderBy,
    t1: &Chunk,
    t2: &Chunk,
) -> Result<Ordering> {
    let mut result = Ordering::Equal;

    for element in &order_by.elements {
        let v1 =
            eval_group_order_by_element(collection_relationships, variables, state, element, t1)?;
        let v2 =
            eval_group_order_by_element(collection_relationships, variables, state, element, t2)?;
        let x = match element.order_direction {
            models::OrderDirection::Asc => compare(v1, v2)?,
            models::OrderDirection::Desc => compare(v2, v1)?,
        };
        result = result.then(x);
    }

    Ok(result)
}
// ANCHOR_END: eval_group_order_by
// ANCHOR: eval_group_order_by_element
fn eval_group_order_by_element(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    element: &models::GroupOrderByElement,
    group: &Chunk,
) -> Result<serde_json::Value> {
    match element.target.clone() {
        models::GroupOrderByTarget::Dimension { index } => {
            group.dimensions.get(index).cloned().ok_or((
                StatusCode::BAD_REQUEST,
                Json(models::ErrorResponse {
                    message: "dimension index out of range".into(),
                    details: serde_json::Value::Null,
                }),
            ))
        }
        models::GroupOrderByTarget::Aggregate { aggregate, path } => {
            let rows = eval_path(
                collection_relationships,
                variables,
                state,
                &path,
                &group.rows,
            )?;
            eval_aggregate(&aggregate, &rows)
        }
    }
}
// ANCHOR_END: eval_group_order_by_element
// ANCHOR: eval_dimensions
fn eval_dimensions(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    row: &Row,
    dimensions: &[ndc_models::Dimension],
) -> Result<Vec<serde_json::Value>> {
    let mut values = vec![];
    for dimension in dimensions {
        let value = eval_dimension(collection_relationships, variables, state, row, dimension)?;
        values.push(value);
    }
    Ok(values)
}
// ANCHOR_END: eval_dimensions
// ANCHOR: eval_dimension
fn eval_dimension(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    row: &Row,
    dimension: &models::Dimension,
) -> Result<serde_json::Value> {
    match dimension {
        models::Dimension::Column {
            column_name,
            field_path,
            path,
        } => eval_column_at_path(
            collection_relationships,
            variables,
            state,
            row,
            path.clone(),
            column_name.clone(),
            field_path.clone(),
        ),
    }
}
// ANCHOR_END: eval_dimension
// ANCHOR: eval_row
fn eval_row(
    fields: &IndexMap<models::FieldName, models::Field>,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    item: &BTreeMap<models::FieldName, serde_json::Value>,
) -> Result<IndexMap<models::FieldName, models::RowFieldValue>> {
    let mut row = IndexMap::new();
    for (field_name, field) in fields {
        row.insert(
            field_name.clone(),
            eval_field(collection_relationships, variables, state, field, item)?,
        );
    }
    Ok(row)
}
// ANCHOR_END: eval_row
// ANCHOR: eval_group_comparison_target
fn eval_group_comparison_target(
    target: &models::GroupComparisonTarget,
    rows: &[Row],
) -> Result<serde_json::Value> {
    match target {
        models::GroupComparisonTarget::Aggregate { aggregate } => eval_aggregate(aggregate, rows),
    }
}
// ANCHOR_END: eval_group_comparison_target
// ANCHOR: eval_aggregates
fn eval_aggregates(
    aggregates: &IndexMap<ndc_models::FieldName, ndc_models::Aggregate>,
    rows: &[Row],
) -> std::result::Result<
    IndexMap<ndc_models::FieldName, serde_json::Value>,
    (StatusCode, Json<ndc_models::ErrorResponse>),
> {
    let mut row: IndexMap<models::FieldName, serde_json::Value> = IndexMap::new();
    for (aggregate_name, aggregate) in aggregates {
        row.insert(aggregate_name.clone(), eval_aggregate(aggregate, rows)?);
    }
    Ok(row)
}
// ANCHOR_END: eval_aggregates
// ANCHOR: eval_aggregate
fn eval_aggregate(aggregate: &models::Aggregate, rows: &[Row]) -> Result<serde_json::Value> {
    match aggregate {
        models::Aggregate::StarCount {} => Ok(serde_json::Value::from(rows.len())),
        models::Aggregate::ColumnCount {
            column,
            field_path,
            distinct,
        } => {
            let values = rows
                .iter()
                .map(|row| eval_column_field_path(row, column, field_path, &BTreeMap::new()))
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
                        message: "unable to encode value".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })
        }
        models::Aggregate::SingleColumn {
            column,
            field_path,
            function,
        } => {
            let values = rows
                .iter()
                .map(|row| eval_column_field_path(row, column, field_path, &BTreeMap::new()))
                .collect::<Result<Vec<_>>>()?;
            eval_aggregate_function(function, values)
        }
    }
}
// ANCHOR_END: eval_aggregate
// ANCHOR: eval_aggregate_function
fn eval_aggregate_function(
    function: &models::AggregateFunctionName,
    values: Vec<serde_json::Value>,
) -> Result<serde_json::Value> {
    let int_values = values
        .iter()
        .map(|value| {
            value
                .as_i64()
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "column is not an integer".into(),
                        details: serde_json::Value::Null,
                    }),
                ))?
                .try_into()
                .map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(models::ErrorResponse {
                            message: "column value out of range".into(),
                            details: serde_json::Value::Null,
                        }),
                    )
                })
        })
        .collect::<Result<Vec<i32>>>()?;
    let agg_value = match function.as_str() {
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
                message: "unable to encode value".into(),
                details: serde_json::Value::Null,
            }),
        )
    })
}
// ANCHOR_END: eval_aggregate_function
// ANCHOR: sort
fn sort(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    collection: Vec<Row>,
    order_by: &Option<models::OrderBy>,
) -> Result<Vec<Row>> {
    match order_by {
        None => Ok(collection),
        Some(order_by) => {
            let mut copy = vec![];
            for item_to_insert in collection {
                let mut index = 0;
                for other in &copy {
                    if let Ordering::Greater = eval_order_by(
                        collection_relationships,
                        variables,
                        state,
                        order_by,
                        other,
                        &item_to_insert,
                    )? {
                        break;
                    }
                    index += 1;
                }
                copy.insert(index, item_to_insert);
            }
            Ok(copy)
        }
    }
}
// ANCHOR_END: sort
// ANCHOR: paginate
fn paginate<I: Iterator>(collection: I, limit: Option<u32>, offset: Option<u32>) -> Vec<I::Item> {
    let start = offset.unwrap_or(0).try_into().unwrap();
    match limit {
        Some(n) => collection.skip(start).take(n.try_into().unwrap()).collect(),
        None => collection.skip(start).collect(),
    }
}
// ANCHOR_END: paginate
// ANCHOR: eval_order_by
fn eval_order_by(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    order_by: &models::OrderBy,
    t1: &Row,
    t2: &Row,
) -> Result<Ordering> {
    let mut result = Ordering::Equal;

    for element in &order_by.elements {
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
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    element: &models::OrderByElement,
    item: &Row,
) -> Result<serde_json::Value> {
    match element.target.clone() {
        models::OrderByTarget::Column {
            name,
            field_path,
            path,
        } => eval_column_at_path(
            collection_relationships,
            variables,
            state,
            item,
            path,
            name,
            field_path,
        ),
        models::OrderByTarget::Aggregate { aggregate, path } => {
            let rows = eval_path(
                collection_relationships,
                variables,
                state,
                &path,
                &[item.clone()],
            )?;
            eval_aggregate(&aggregate, &rows)
        }
    }
}
// ANCHOR_END: eval_order_by_element
// ANCHOR: eval_column_field_path
fn eval_column_field_path(
    row: &Row,
    column_name: &models::FieldName,
    field_path: &Option<Vec<models::FieldName>>,
    arguments: &BTreeMap<models::ArgumentName, models::Argument>,
) -> Result<serde_json::Value> {
    let column_value = eval_column(&BTreeMap::default(), row, column_name, arguments)?;
    match field_path {
        None => Ok(column_value),
        Some(path) => eval_field_path(path, column_value),
    }
}
// ANCHOR_END: eval_column_field_path
// ANCHOR: eval_field_path
fn eval_field_path(
    path: &[ndc_models::FieldName],
    value: serde_json::Value,
) -> Result<serde_json::Value> {
    path.iter()
        .try_fold(&value, |value, field_name| value.get(field_name.as_str()))
        .cloned()
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(models::ErrorResponse {
                message: "invalid field path".into(),
                details: serde_json::Value::Null,
            }),
        ))
}
// ANCHOR_END: eval_field_path
// ANCHOR: eval_column_at_path
fn eval_column_at_path(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    item: &Row,
    path: Vec<models::PathElement>,
    name: models::FieldName,
    field_path: Option<Vec<models::FieldName>>,
) -> Result<serde_json::Value> {
    let rows: Vec<Row> = eval_path(
        collection_relationships,
        variables,
        state,
        &path,
        &[item.clone()],
    )?;
    if rows.len() > 1 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(models::ErrorResponse {
                message: "path elements used in sorting and grouping cannot yield multiple rows"
                    .into(),
                details: serde_json::Value::Null,
            }),
        ));
    }
    match rows.first() {
        Some(row) => eval_column_field_path(row, &name, &field_path, &BTreeMap::new()),
        None => Ok(serde_json::Value::Null),
    }
}
// ANCHOR_END: eval_column_at_path
// ANCHOR: eval_path
fn eval_path(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    path: &[models::PathElement],
    items: &[Row],
) -> Result<Vec<Row>> {
    let mut result: Vec<Row> = items.to_vec();

    for path_element in path {
        let relationship = collection_relationships
            .get(&path_element.relationship)
            .ok_or((
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
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    relationship: &models::Relationship,
    arguments: &BTreeMap<models::ArgumentName, models::RelationshipArgument>,
    source: &[Row],
    predicate: &Option<Box<models::Expression>>,
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

    for src_row in source {
        let mut all_arguments = BTreeMap::new();

        for (argument_name, argument_value) in &relationship.arguments {
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

        for (argument_name, argument_value) in arguments {
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

        let target =
            get_collection_by_name(&relationship.target_collection, &all_arguments, state)?;

        for tgt_row in &target {
            if eval_column_mapping(relationship, src_row, tgt_row)?
                && if let Some(expression) = predicate {
                    eval_expression(
                        collection_relationships,
                        variables,
                        state,
                        expression,
                        &[],
                        tgt_row,
                    )?
                } else {
                    true
                }
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
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    argument: &models::Argument,
) -> Result<serde_json::Value> {
    match argument {
        models::Argument::Variable { name } => {
            let value = variables
                .get(name)
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
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    row: &Row,
    argument: &models::RelationshipArgument,
) -> Result<serde_json::Value> {
    match argument {
        models::RelationshipArgument::Variable { name } => {
            let value = variables
                .get(name)
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
        models::RelationshipArgument::Column { name } => {
            eval_column(&BTreeMap::default(), row, name, &BTreeMap::default())
        }
    }
}
// ANCHOR_END: eval_relationship_argument
// ANCHOR: eval_expression
// ANCHOR: eval_expression_signature
fn eval_expression(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    expr: &models::Expression,
    scopes: &[&Row],
    item: &Row,
) -> Result<bool> {
    // ANCHOR_END: eval_expression_signature
    // ANCHOR: eval_expression_logical
    match expr {
        models::Expression::And { expressions } => {
            for expr in expressions {
                if !eval_expression(
                    collection_relationships,
                    variables,
                    state,
                    expr,
                    scopes,
                    item,
                )? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        models::Expression::Or { expressions } => {
            for expr in expressions {
                if eval_expression(
                    collection_relationships,
                    variables,
                    state,
                    expr,
                    scopes,
                    item,
                )? {
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
                scopes,
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
                    item,
                )?;
                Ok(vals.is_null())
            }
        },
        // ANCHOR_END: eval_expression_unary_operators
        // ANCHOR: eval_expression_binary_operators
        models::Expression::BinaryComparisonOperator {
            column,
            operator,
            value,
        } => {
            let left_val =
                eval_comparison_target(collection_relationships, variables, state, column, item)?;
            let right_vals = eval_comparison_value(
                collection_relationships,
                variables,
                value,
                state,
                scopes,
                item,
            )?;
            eval_comparison_operator(operator, &left_val, right_vals)
        }
        // ANCHOR_END: eval_expression_binary_operators
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
                predicate: predicate.clone().map(|e| *e),
                groups: None,
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
                Root::PushCurrentRow(scopes),
                collection,
            )?;
            let rows: Vec<IndexMap<_, _>> = row_set.rows.ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(models::ErrorResponse {
                    message: "expected 'rows'".into(),
                    details: serde_json::Value::Null,
                }),
            ))?;
            Ok(!rows.is_empty())
        } // ANCHOR_END: eval_expression_exists
    }
}
// ANCHOR_END: eval_expression
// ANCHOR: eval_comparison_operator
fn eval_comparison_operator(
    operator: &models::ComparisonOperatorName,
    left_val: &serde_json::Value,
    right_vals: Vec<serde_json::Value>,
) -> std::prelude::v1::Result<bool, (StatusCode, Json<models::ErrorResponse>)> {
    match operator.as_str() {
        "eq" => {
            for right_val in &right_vals {
                if left_val == right_val {
                    return Ok(true);
                }
            }

            Ok(false)
        }
        // ANCHOR: eval_expression_custom_binary_operators
        "like" => {
            for regex_val in &right_vals {
                let column_str = left_val.as_str().ok_or((
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "regex is not a string".into(),
                        details: serde_json::Value::Null,
                    }),
                ))?;
                let regex_str = regex_val.as_str().ok_or((
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "regex is invalid".into(),
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

            Ok(false)
        }
        // ANCHOR_END: eval_expression_custom_binary_operators
        // ANCHOR: eval_expression_binary_array_operators
        "in" => {
            for comparison_value in &right_vals {
                let right_vals = comparison_value.as_array().ok_or((
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "expected array".into(),
                        details: serde_json::Value::Null,
                    }),
                ))?;

                for right_val in right_vals {
                    if left_val == right_val {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }
        // ANCHOR_END: eval_expression_binary_array_operators
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(models::ErrorResponse {
                message: "unknown binary comparison operator".into(),
                details: serde_json::Value::Null,
            }),
        )),
    }
}
// ANCHOR_END: eval_comparison_operator
// ANCHOR: eval_in_collection
fn eval_in_collection(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    item: &Row,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    in_collection: &models::ExistsInCollection,
) -> Result<Vec<Row>> {
    match in_collection {
        models::ExistsInCollection::Related {
            relationship,
            arguments,
        } => {
            let relationship = collection_relationships.get(relationship).ok_or((
                StatusCode::BAD_REQUEST,
                Json(models::ErrorResponse {
                    message: "relationship is undefined".into(),
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
                &None,
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

            get_collection_by_name(collection, &arguments, state)
        }
        ndc_models::ExistsInCollection::NestedCollection {
            column_name,
            field_path,
            arguments,
        } => {
            let value =
                eval_column_field_path(item, column_name, &Some(field_path.clone()), arguments)?;
            serde_json::from_value(value).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "nested collection must be an array of objects".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })
        }
    }
}
// ANCHOR_END: eval_in_collection
// ANCHOR: eval_comparison_target
fn eval_comparison_target(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    target: &models::ComparisonTarget,
    item: &Row,
) -> Result<serde_json::Value> {
    match target {
        models::ComparisonTarget::Column { name, field_path } => {
            eval_column_field_path(item, name, field_path, &BTreeMap::new())
        }
        models::ComparisonTarget::Aggregate { aggregate, path } => {
            let rows: Vec<Row> = eval_path(
                collection_relationships,
                variables,
                state,
                path,
                &[item.clone()],
            )?;
            eval_aggregate(aggregate, &rows)
        }
    }
}
// ANCHOR_END: eval_comparison_target
// ANCHOR: eval_column
fn eval_column(
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    row: &Row,
    column_name: &models::FieldName,
    arguments: &BTreeMap<models::ArgumentName, models::Argument>,
) -> Result<serde_json::Value> {
    let column = row.get(column_name).cloned().ok_or((
        StatusCode::BAD_REQUEST,
        Json(models::ErrorResponse {
            message: "invalid column name".into(),
            details: serde_json::Value::Null,
        }),
    ))?;

    if let Some(array) = column.as_array() {
        let limit_argument = arguments.get("limit").ok_or((
            StatusCode::BAD_REQUEST,
            Json(models::ErrorResponse {
                message: format!("Expected argument 'limit' in column {column_name}"),
                details: serde_json::Value::Null,
            }),
        ))?;
        let limit =
            serde_json::from_value::<Option<usize>>(eval_argument(variables, limit_argument)?)
                .map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(models::ErrorResponse {
                            message: "limit must be null or an integer".into(),
                            details: serde_json::Value::Null,
                        }),
                    )
                })?;

        let result_array = array[0..limit.unwrap_or(array.len())].to_vec();

        Ok(serde_json::Value::Array(result_array))
    } else {
        Ok(column)
    }
}
// ANCHOR_END: eval_column
// ANCHOR: eval_comparison_value
fn eval_comparison_value(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    comparison_value: &models::ComparisonValue,
    state: &AppState,
    scopes: &[&Row],
    item: &Row,
) -> Result<Vec<serde_json::Value>> {
    match comparison_value {
        models::ComparisonValue::Column {
            name,
            field_path,
            path,
            scope,
        } => {
            let scope = scope.map_or(Ok(item), |scope| {
                if scope == 0 {
                    Ok(item)
                } else {
                    Ok(*scopes.get(scopes.len() - 1 - scope).ok_or((
                        StatusCode::BAD_REQUEST,
                        Json(models::ErrorResponse {
                            message: "named scope is invalid".into(),
                            details: serde_json::Value::Null,
                        }),
                    ))?)
                }
            })?;

            let items = eval_path(
                collection_relationships,
                variables,
                state,
                path,
                &[scope.clone()],
            )?;

            items
                .iter()
                .map(|item| eval_column_field_path(item, name, field_path, &BTreeMap::new()))
                .collect()
        }
        models::ComparisonValue::Scalar { value } => Ok(vec![value.clone()]),
        models::ComparisonValue::Variable { name } => {
            let value = variables
                .get(name)
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
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    value: serde_json::Value,
    nested_field: &models::NestedField,
) -> Result<models::RowFieldValue> {
    match nested_field {
        models::NestedField::Object(nested_object) => {
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
                &nested_object.fields,
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
            let array: Vec<serde_json::Value> = serde_json::from_value(value).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(models::ErrorResponse {
                        message: "Expected array".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })?;

            let result_array = array
                .into_iter()
                .map(|value| {
                    eval_nested_field(
                        collection_relationships,
                        variables,
                        state,
                        value.clone(),
                        fields,
                    )
                })
                .collect::<Result<Vec<_>>>()?;
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
        ndc_models::NestedField::Collection(models::NestedCollection { query }) => {
            let collection = serde_json::from_value::<Vec<Row>>(value).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(models::ErrorResponse {
                        message: "cannot decode rows".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })?;

            let row_set = execute_query(
                collection_relationships,
                variables,
                state,
                query,
                Root::Reset,
                collection,
            )?;

            let row_set_json = serde_json::to_value(row_set).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(models::ErrorResponse {
                        message: "cannot encode rowset".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })?;

            Ok(models::RowFieldValue(row_set_json))
        }
    }
}
// ANCHOR_END: eval_nested_field
// ANCHOR: eval_field
fn eval_field(
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    variables: &BTreeMap<models::VariableName, serde_json::Value>,
    state: &AppState,
    field: &models::Field,
    item: &Row,
) -> Result<models::RowFieldValue> {
    match field {
        models::Field::Column {
            column,
            fields,
            arguments,
        } => {
            let col_val = eval_column(variables, item, column, arguments)?;
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
            let relationship = collection_relationships.get(relationship).ok_or((
                StatusCode::BAD_REQUEST,
                Json(models::ErrorResponse {
                    message: "relationship is undefined".into(),
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
                &None,
            )?;
            let row_set = execute_query(
                collection_relationships,
                variables,
                state,
                query,
                Root::Reset,
                collection,
            )?;
            let row_set_json = serde_json::to_value(row_set).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(models::ErrorResponse {
                        message: "cannot encode rowset".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })?;
            Ok(models::RowFieldValue(row_set_json))
        }
    }
}
// ANCHOR_END: eval_field
// ANCHOR: query_explain
async fn post_query_explain(
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
// ANCHOR_END: query_explain
// ANCHOR: mutation_explain
async fn post_mutation_explain(
    Json(_request): Json<models::MutationRequest>,
) -> Result<Json<models::ExplainResponse>> {
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(models::ErrorResponse {
            message: "explain is not supported".into(),
            details: serde_json::Value::Null,
        }),
    ))
}
// ANCHOR_END: mutation_explain
// ANCHOR: post_mutation_signature
async fn post_mutation(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(request): Json<models::MutationRequest>,
) -> Result<Json<models::MutationResponse>> {
    // ANCHOR_END: post_mutation_signature
    // ANCHOR: post_mutation
    if request.operations.len() > 1 {
        Err((
            StatusCode::NOT_IMPLEMENTED,
            Json(models::ErrorResponse {
                message: "transactional mutations are not supported".into(),
                details: serde_json::Value::Null,
            }),
        ))
    } else {
        let mut state = state.lock().await;

        let mut operation_results = vec![];

        for operation in &request.operations {
            let operation_result = execute_mutation_operation(
                &mut state,
                &request.collection_relationships,
                operation,
            )?;
            operation_results.push(operation_result);
        }

        Ok(Json(models::MutationResponse { operation_results }))
    }
}
// ANCHOR_END: post_mutation
// ANCHOR: execute_mutation_operation
fn execute_mutation_operation(
    state: &mut AppState,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
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
    name: &models::ProcedureName,
    arguments: &BTreeMap<models::ArgumentName, serde_json::Value>,
    fields: &Option<models::NestedField>,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
) -> std::result::Result<models::MutationOperationResults, (StatusCode, Json<models::ErrorResponse>)>
// ANCHOR_END: execute_procedure_signature
// ANCHOR: execute_procedure_signature_impl
{
    match name.as_str() {
        "upsert_article" => {
            execute_upsert_article(state, arguments, fields, collection_relationships)
        }
        "delete_articles" => {
            execute_delete_articles(state, arguments, fields, collection_relationships)
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
    arguments: &BTreeMap<models::ArgumentName, serde_json::Value>,
    fields: &Option<models::NestedField>,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
) -> std::result::Result<models::MutationOperationResults, (StatusCode, Json<models::ErrorResponse>)>
{
    let article = arguments.get("article").ok_or((
        StatusCode::BAD_REQUEST,
        Json(models::ErrorResponse {
            message: "Expected argument 'article'".into(),
            details: serde_json::Value::Null,
        }),
    ))?;
    let article_obj: Row = serde_json::from_value(article.clone()).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(models::ErrorResponse {
                message: "article must be an object".into(),
                details: serde_json::Value::Null,
            }),
        )
    })?;
    let id = article_obj.get("id").ok_or((
        StatusCode::BAD_REQUEST,
        Json(models::ErrorResponse {
            message: "article missing field 'id'".into(),
            details: serde_json::Value::Null,
        }),
    ))?;
    let id_int = id
        .as_i64()
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(models::ErrorResponse {
                message: "id must be an integer".into(),
                details: serde_json::Value::Null,
            }),
        ))?
        .try_into()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(models::ErrorResponse {
                    message: "id out of range".into(),
                    details: serde_json::Value::Null,
                }),
            )
        })?;
    let old_row = state.articles.insert(id_int, article_obj);

    Ok(models::MutationOperationResults::Procedure {
        result: old_row.map_or(Ok(serde_json::Value::Null), |old_row| {
            let old_row_value = serde_json::to_value(old_row).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(models::ErrorResponse {
                        message: "cannot encode response".into(),
                        details: serde_json::Value::Null,
                    }),
                )
            })?;

            let old_row_fields = match fields {
                None => Ok(models::RowFieldValue(old_row_value)),
                Some(nested_field) => eval_nested_field(
                    collection_relationships,
                    &BTreeMap::new(),
                    state,
                    old_row_value,
                    nested_field,
                ),
            }?;

            Ok(old_row_fields.0)
        })?,
    })
}
// ANCHOR_END: execute_upsert_article
// ANCHOR: execute_delete_articles
fn execute_delete_articles(
    state: &mut AppState,
    arguments: &BTreeMap<models::ArgumentName, serde_json::Value>,
    fields: &Option<models::NestedField>,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
) -> std::result::Result<models::MutationOperationResults, (StatusCode, Json<models::ErrorResponse>)>
{
    let predicate_value = arguments.get("where").ok_or((
        StatusCode::BAD_REQUEST,
        Json(models::ErrorResponse {
            message: "Expected argument 'where'".into(),
            details: serde_json::Value::Null,
        }),
    ))?;
    let predicate: models::Expression =
        serde_json::from_value(predicate_value.clone()).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(models::ErrorResponse {
                    message: "Bad predicate".into(),
                    details: serde_json::Value::Null,
                }),
            )
        })?;

    let mut removed: Vec<Row> = vec![];

    let state_snapshot = state.clone();

    for article in state.articles.values_mut() {
        if eval_expression(
            &BTreeMap::new(),
            &BTreeMap::new(),
            &state_snapshot,
            &predicate,
            &[],
            article,
        )? {
            removed.push(article.clone());
        }
    }

    let removed_value = serde_json::to_value(removed).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(models::ErrorResponse {
                message: "cannot encode response".into(),
                details: serde_json::Value::Null,
            }),
        )
    })?;

    let removed_fields = match fields {
        None => Ok(models::RowFieldValue(removed_value)),
        Some(nested_field) => eval_nested_field(
            collection_relationships,
            &BTreeMap::new(),
            &state_snapshot,
            removed_value,
            nested_field,
        ),
    }?;

    Ok(models::MutationOperationResults::Procedure {
        result: removed_fields.0,
    })
}
// ANCHOR_END: execute_delete_articles

fn eval_column_mapping(
    relationship: &models::Relationship,
    src_row: &Row,
    tgt_row: &Row,
) -> Result<bool> {
    for (src_column, tgt_column) in &relationship.column_mapping {
        let src_value = eval_column(
            &BTreeMap::default(),
            src_row,
            src_column,
            &BTreeMap::default(),
        )?;
        let tgt_value = eval_column(
            &BTreeMap::default(),
            tgt_row,
            tgt_column,
            &BTreeMap::default(),
        )?;
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
    use ndc_models as models;
    use ndc_test::{
        configuration::{TestConfiguration, TestGenerationConfiguration, TestOptions},
        connector::Connector,
        error::Error,
        reporter::TestResults,
        test_cases::query::validate::validate_response,
        test_connector,
    };
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

            write!(expected, "{response_json}").unwrap();

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
            let schema = crate::get_schema().await;

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

                let path = entry.path();
                let test_name = path.file_name().unwrap().to_str().unwrap();

                let expected_path = { PathBuf::from_iter(["query", test_name, "expected.json"]) };

                let state = Arc::new(Mutex::new(crate::init_app_state()));
                let response = crate::post_query(State(state), Json(request.clone()))
                    .await
                    .unwrap();

                validate_response(&schema, &request, &response)
                    .unwrap_or_else(|_| panic!("unable to validate response in test {test_name}"));

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
        state: crate::AppState,
    }

    #[async_trait(?Send)]
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
            Ok(post_query(
                State(Arc::new(Mutex::new(self.state.clone()))),
                Json(request),
            )
            .await
            .map_err(|(_, Json(err))| Error::ConnectorError(err))?
            .0)
        }

        async fn mutation(
            &self,
            request: models::MutationRequest,
        ) -> Result<models::MutationResponse, Error> {
            Ok(post_mutation(
                State(Arc::new(Mutex::new(self.state.clone()))),
                Json(request),
            )
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
                gen_config: TestGenerationConfiguration::default(),
                options: TestOptions::default(),
            };
            let connector = Reference {
                state: init_app_state(),
            };
            let mut reporter = TestResults::default();
            test_connector(&configuration, &connector, &mut reporter).await;
            assert!(reporter.failures.is_empty());
        });
    }
}
