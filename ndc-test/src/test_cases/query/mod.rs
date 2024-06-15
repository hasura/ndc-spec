mod aggregates;
mod relationships;
mod simple_queries;

mod common;
mod context;
pub mod function;
pub mod validate;

use crate::configuration::{
    FixtureConfiguration, FixtureOperationType, TestGenerationConfiguration,
};
use crate::connector::Connector;
use crate::reporter::Reporter;
use crate::test_cases::fixture::write_fixture_files;
use crate::{nest, test};

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

                Some(())
            }
        });
    }
}

pub async fn make_query_fixtures<R: Reporter>(
    config: &FixtureConfiguration,
    reporter: &mut R,
    schema: &models::SchemaResponse,
    rng: &mut SmallRng,
) {
    if config.operation_types.is_empty()
        || config
            .operation_types
            .contains(&FixtureOperationType::Collection)
    {
        nest!("Collection", reporter, async {
            for collection_info in &schema.collections {
                if !config.operations.is_empty()
                    && !config.operations.contains(&collection_info.name)
                {
                    continue;
                }

                test!(collection_info.name.as_str(), reporter, {
                    async {
                        let (request, response) = simple_queries::make_select_top_n_rows_fixture(
                            &config.gen_config,
                            rng,
                            schema,
                            collection_info,
                        );
                        let snapshot_subdir = {
                            let mut builder = config.snapshots_dir.clone();
                            builder.extend(vec!["query", collection_info.name.as_str()]);
                            builder
                        };
                        write_fixture_files(snapshot_subdir, request, response)
                    }
                });
            }
        });
    }

    if config.operation_types.is_empty()
        || config
            .operation_types
            .contains(&FixtureOperationType::Function)
    {
        nest!("Function", reporter, async {
            for function_info in &schema.functions {
                if !config.operations.is_empty() && !config.operations.contains(&function_info.name)
                {
                    continue;
                }

                test!(function_info.name.as_str(), reporter, {
                    async {
                        let (request, response) = function::make_function_fixture(
                            &config.gen_config,
                            rng,
                            schema,
                            function_info,
                        )?;
                        let snapshot_subdir = {
                            let mut builder = config.snapshots_dir.clone();
                            builder.extend(vec!["query", function_info.name.as_str()]);
                            builder
                        };
                        write_fixture_files(snapshot_subdir, request, response)
                    }
                });
            }
        });
    }
}
