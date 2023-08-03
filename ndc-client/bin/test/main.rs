use std::collections::BTreeMap;

use clap::Parser;
use indexmap::IndexMap;
use ndc_client::apis::configuration::Configuration;
use ndc_client::apis::default_api as api;
use ndc_client::models;

#[derive(Parser)]
struct Options {
    #[arg(long, value_name = "ENDPOINT")]
    endpoint: String,
}

#[tokio::main]
async fn main() {
    let options = Options::parse();

    let http_client = reqwest::Client::new();

    let configuration = Configuration {
        base_path: options.endpoint,
        user_agent: None,
        client: http_client.clone(),
        basic_auth: None,
        oauth_access_token: None,
        bearer_access_token: None,
        api_key: None,
    };

    println!("Fetching /capabilities");
    let capabilities = api::capabilities_get(&configuration).await.unwrap();

    println!("Validating capabilities");
    validate_capabilities(&capabilities);

    print!("Fetching /schema");
    let schema = api::schema_get(&configuration).await.unwrap();

    println!("Validating schema");
    validate_schema(&schema);

    println!("Testing /query");
    test_query(&configuration, &capabilities, &schema).await;
}

fn validate_capabilities(_capabilities: &models::CapabilitiesResponse) {
    // TODO: validate capabilities.version
}

fn validate_schema(schema: &models::SchemaResponse) {
    println!("Validating object_types");
    for (_type_name, object_type) in schema.object_types.iter() {
        for (_field_name, object_field) in object_type.fields.iter() {
            validate_type(schema, &object_field.r#type);
            for (_arg_name, arg_info) in object_field.arguments.iter() {
                validate_type(schema, &arg_info.argument_type);
            }
        }
    }

    println!("Validating collections");
    for collection_info in schema.collections.iter() {
        println!("Validating collection {}", collection_info.name);
        let collection_type = schema
            .object_types
            .get(collection_info.collection_type.as_str());

        for (_arg_name, arg_info) in collection_info.arguments.iter() {
            validate_type(schema, &arg_info.argument_type);
        }

        match collection_type {
            None => {
                panic!(
                    "collection type {} is not a defined object type",
                    collection_info.collection_type
                );
            }

            Some(collection_type) => {
                println!("Validating columns");
                if let Some(insertable_columns) = &collection_info.insertable_columns {
                    for insertable_column in insertable_columns.iter() {
                        assert!(
                            collection_type
                                .fields
                                .contains_key(insertable_column.as_str()),
                            "insertable column {} is not defined on collection type",
                            insertable_column
                        );
                    }
                }
                if let Some(updatable_columns) = &collection_info.updatable_columns {
                    for updatable_column in updatable_columns.iter() {
                        assert!(
                            collection_type
                                .fields
                                .contains_key(updatable_column.as_str()),
                            "updatable column {} is not defined on collection type",
                            updatable_column
                        );
                    }
                }
            }
        };
    }

    println!("Validating functions");
    for function_info in schema.functions.iter() {
        println!("Validating function {}", function_info.name);
        validate_type(schema, &function_info.result_type);

        for (_arg_name, arg_info) in function_info.arguments.iter() {
            validate_type(schema, &arg_info.argument_type);
        }
    }

    println!("Validating procedures");
    for procedure_info in schema.procedures.iter() {
        println!("Validating procedure {}", procedure_info.name);

        validate_type(schema, &procedure_info.result_type);

        for (_arg_name, arg_info) in procedure_info.arguments.iter() {
            validate_type(schema, &arg_info.argument_type);
        }
    }
}

fn validate_type(schema: &models::SchemaResponse, r#type: &models::Type) {
    match r#type {
        models::Type::Named { name } => {
            assert!(
                schema.object_types.contains_key(name.as_str())
                    || schema.scalar_types.contains_key(name.as_str()),
                "named type {} is not a defined object or scalar type",
                name
            );
        }
        models::Type::Array { element_type } => {
            validate_type(schema, element_type);
        }
        models::Type::Nullable { underlying_type } => {
            validate_type(schema, underlying_type);
        }
    }
}

async fn test_query(
    configuration: &Configuration,
    _capabilities: &models::CapabilitiesResponse,
    schema: &models::SchemaResponse,
) {
    println!("Testing simple queries");
    for collection_info in schema.collections.iter() {
        println!("Querying collection {}", collection_info.name);
        test_simple_queries(configuration, schema, collection_info).await;
    }

    println!("Testing aggregate queries");
    for collection_info in schema.collections.iter() {
        println!("Querying collection {}", collection_info.name);
        test_aggregate_queries(configuration, schema, collection_info).await;
    }
}

async fn test_simple_queries(
    configuration: &Configuration,
    schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
) {
    let collection_type = schema
        .object_types
        .get(collection_info.collection_type.as_str())
        .unwrap();
    let fields = collection_type
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
        .collect::<IndexMap<String, models::Field>>();
    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: None,
            fields: Some(fields),
            limit: Some(10),
            offset: None,
            order_by: None,
            predicate: None,
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };
    let _response = api::query_post(configuration, query_request).await;

    // TODO: assert the response matches the type
}

async fn test_aggregate_queries(
    configuration: &Configuration,
    _schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
) {
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
    if let [row_set] = &*response.0.clone() {
        assert!(
            row_set.rows.is_none(),
            "aggregate-only query should not return rows"
        );
        if let Some(aggregates) = &row_set.aggregates {
            assert!(
                aggregates.contains_key("count"),
                "aggregate query should return requested count aggregate"
            );
        } else {
            panic!("aggregate query should return aggregates");
        }
    } else {
        panic!("response should return a single rowset");
    }
}
