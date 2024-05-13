mod capabilities;
pub mod query;
mod schema;

use crate::configuration::TestGenerationConfiguration;
use crate::connector::Connector;
use crate::nest;
use crate::reporter::Reporter;

use rand::rngs::SmallRng;

pub async fn run_all_tests<C: Connector, R: Reporter>(
    gen_config: &TestGenerationConfiguration,
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

    nest!("Query", reporter, {
        query::test_query(gen_config, connector, reporter, &capabilities, &schema, rng)
    });

    Some(())
}
