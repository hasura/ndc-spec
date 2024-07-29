mod aggregates;
mod grouping;
mod relationships;
mod simple_queries;

mod common;
mod context;
pub mod validate;

use crate::configuration::TestGenerationConfiguration;
use crate::connector::Connector;
use crate::nest;
use crate::reporter::Reporter;

use ndc_models as models;
use rand::rngs::SmallRng;

pub async fn test_query<C: Connector, R: Reporter>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    reporter: &mut R,
    capabilities: &models::CapabilitiesResponse,
    schema: &models::SchemaResponse,
    rng: &mut SmallRng,
) {
    for collection_info in &schema.collections {
        nest!(collection_info.name.as_str(), reporter, {
            async {
                if collection_info.arguments.is_empty() {
                    let context = nest!("Simple queries", reporter, {
                        simple_queries::test_simple_queries(
                            gen_config,
                            connector,
                            reporter,
                            rng,
                            schema,
                            collection_info,
                        )
                    })?;

                    if capabilities.capabilities.relationships.is_some() {
                        nest!("Relationship queries", reporter, {
                            relationships::test_relationship_queries(
                                gen_config,
                                connector,
                                reporter,
                                schema,
                                collection_info,
                                &context,
                                rng,
                            )
                        });
                    }

                    if let Some(aggregates) = &capabilities.capabilities.query.aggregates {
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

                        if aggregates.group_by.is_some() {
                            nest!("Grouping queries", reporter, {
                                grouping::test_grouping(
                                    gen_config,
                                    connector,
                                    reporter,
                                    schema,
                                    collection_info,
                                    rng,
                                )
                            });
                        }
                    }
                } else {
                    eprintln!("Skipping parameterized collection {}", collection_info.name);
                }

                Some(())
            }
        });
    }
}
