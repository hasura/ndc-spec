mod aggregates;
mod relationships;
mod simple_queries;

mod common;
mod context;
mod expectations;

use std::cell::RefCell;

use crate::connector::Connector;
use crate::reporter::{Reporter, ReporterExt};
use crate::results::TestResults;

use ndc_client::models;
use rand::rngs::SmallRng;

pub async fn test_query<C: Connector, R: Reporter>(
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
                            aggregates::test_aggregate_queries(
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
