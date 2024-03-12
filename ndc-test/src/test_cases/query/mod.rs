mod aggregates;
mod relationships;
mod simple_queries;

mod common;
mod context;
mod validate;

use crate::configuration::TestGenerationConfiguration;
use crate::connector::Connector;
use crate::nest;
use crate::reporter::Reporter;

use ndc_client::models;
use rand::rngs::SmallRng;

pub async fn test_query<C: Connector, R: Reporter>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    reporter: &mut R,
    capabilities: &models::CapabilitiesResponse,
    schema: &models::SchemaResponse,
    rng: &mut SmallRng,
) {
    for collection_info in schema.collections.iter() {
        nest!(collection_info.name.as_str(), reporter, {
            async {
                if collection_info.arguments.is_empty() {
                    nest!("Simple queries", reporter, {
                        simple_queries::test_simple_queries(
                            gen_config,
                            connector,
                            reporter,
                            rng,
                            schema,
                            collection_info,
                        )
                    });

                    if capabilities.capabilities.relationships.is_some() {
                        nest!("Relationship queries", reporter, {
                            relationships::test_relationship_queries(
                                gen_config,
                                connector,
                                reporter,
                                schema,
                                collection_info,
                                rng,
                            )
                        });
                    }

                    if capabilities.capabilities.query.aggregates.is_some() {
                        nest!("Aggregate queries", reporter, {
                            aggregates::test_aggregate_queries(
                                gen_config,
                                connector,
                                reporter,
                                schema,
                                collection_info,
                                rng,
                            )
                        });
                    }
                } else {
                    eprintln!("Skipping parameterized collection {}", collection_info.name);
                }
            }
        });
    }
}
