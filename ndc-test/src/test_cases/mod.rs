mod capabilities;
mod query;
mod schema;

use crate::connector::Connector;
use crate::nest;
use crate::reporter::Reporter;

use rand::rngs::SmallRng;

pub async fn run_all_tests<C: Connector, R: Reporter>(
    connector: &C,
    reporter: &mut R,
    rng: &mut SmallRng,
) -> Option<()> {
    let capabilities = nest!("Capabilities", reporter, {
        capabilities::test_capabilities(connector, reporter)
    })
    .await?;

    let schema = nest!("Schema", reporter, {
        schema::test_schema(connector, reporter)
    })
    .await?;

    nest!("Query", reporter, {
        query::test_query(connector, reporter, &capabilities, &schema, rng)
    })
    .await;

    Some(())
}
