mod relationships;
mod simple_queries;

mod common;
mod context;
mod expectations;
mod snapshot;

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::configuration::TestConfiguration;
use crate::connector::Connector;
use crate::error::Error;
use crate::error::Result;
use crate::reporter::{Reporter, ReporterExt};
use crate::results::TestResults;

use indexmap::IndexMap;

use ndc_client::models;
use rand::rngs::SmallRng;

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
                            simple_queries::test_simple_queries(
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
                                relationships::test_relationship_queries(
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
        .get(collection_info.collection_type.as_str())?;

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
) -> Result<u64> {
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
    let response =
        snapshot::execute_and_snapshot_query(configuration, connector, query_request).await?;
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
) -> Result<()> {
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
    let response =
        snapshot::execute_and_snapshot_query(configuration, connector, query_request).await?;
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
