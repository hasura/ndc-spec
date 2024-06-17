mod procedure;

use rand::rngs::SmallRng;

use crate::{
    configuration, connector::Connector, nest, reporter::Reporter, test, test_cases::fixture,
};

/// Generate mutation fixture for replay tests
pub async fn make_mutation_fixtures<C: Connector, R: Reporter>(
    config: &configuration::FixtureConfiguration,
    connector: &C,
    reporter: &mut R,
    schema: &ndc_models::SchemaResponse,
    rng: &mut SmallRng,
) {
    if config.operation_types.is_empty()
        || config
            .operation_types
            .contains(&configuration::FixtureOperationType::Procedure)
    {
        nest!("Procedure", reporter, async {
            for procedure_info in &schema.procedures {
                if !config.operations.is_empty()
                    && !config.operations.contains(&procedure_info.name)
                {
                    continue;
                }

                let (snapshot_subdir, writable) = fixture::eval_snapshot_directory(
                    config.snapshots_dir.clone(),
                    vec!["mutation", procedure_info.name.as_str()],
                    config.write_mode.clone(),
                );

                if !writable {
                    continue;
                }
                test!(procedure_info.name.as_str(), reporter, {
                    async {
                        let (request, response) = procedure::make_procedure_fixture(
                            &config.gen_config,
                            connector,
                            rng,
                            schema,
                            procedure_info,
                        )
                        .await?;
                        fixture::write_fixture_files(snapshot_subdir, request, response)
                    }
                });
            }
        });
    }
}
