mod capabilities;
pub mod query;
mod schema;

use crate::configuration::{TestGenerationConfiguration, TestOptions};
use crate::connector::Connector;
use crate::nest;
use crate::reporter::Reporter;
use crate::test_cases::query::validate::ValidatingConnector;

use rand::rngs::SmallRng;

pub async fn run_all_tests<C: Connector, R: Reporter>(
    gen_config: &TestGenerationConfiguration,
    options: &TestOptions,
    connector: &C,
    reporter: &mut R,
    rng: &mut SmallRng,
) -> Option<()> {
    let capabilities = nest!("Capabilities", reporter, {
        capabilities::test_capabilities(connector, reporter)
    })?;

    let schema = nest!("Schema", reporter, {
        schema::test_schema(connector, reporter)
    })?;

    nest!("Query", reporter, async {
        if options.validate_responses {
            query::test_query(
                gen_config,
                &ValidatingConnector {
                    connector,
                    schema: &schema,
                },
                reporter,
                &capabilities,
                &schema,
                rng,
            )
            .await;
        } else {
            query::test_query(gen_config, connector, reporter, &capabilities, &schema, rng).await;
        }
    });

    Some(())
}
