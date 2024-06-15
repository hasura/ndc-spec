mod procedure;

use rand::rngs::SmallRng;

use crate::{
    configuration::{FixtureConfiguration, FixtureOperationType},
    nest,
    reporter::Reporter,
    test,
    test_cases::fixture::write_fixture_files,
};

/// Generate mutation fixture for replay tests
pub async fn make_mutation_fixtures<R: Reporter>(
    config: &FixtureConfiguration,
    reporter: &mut R,
    schema: &ndc_models::SchemaResponse,
    rng: &mut SmallRng,
) {
    if config.operation_types.is_empty()
        || config
            .operation_types
            .contains(&FixtureOperationType::Procedure)
    {
        nest!("Procedure", reporter, async {
            for procedure_info in &schema.procedures {
                if !config.operations.is_empty()
                    && !config.operations.contains(&procedure_info.name)
                {
                    continue;
                }

                test!(procedure_info.name.as_str(), reporter, {
                    async {
                        let (request, response) = procedure::make_procedure_fixture(
                            &config.gen_config,
                            rng,
                            schema,
                            procedure_info,
                        );
                        let snapshot_subdir = {
                            let mut builder = config.snapshots_dir.clone();
                            builder.extend(vec!["mutation", procedure_info.name.as_str()]);
                            builder
                        };
                        write_fixture_files(snapshot_subdir, request, response)
                    }
                });
            }
        });
    }
}
