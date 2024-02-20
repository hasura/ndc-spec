mod capabilities;
mod query;
mod schema;

use std::cell::RefCell;

use crate::configuration::TestConfiguration;
use crate::connector::Connector;
use crate::reporter::{Reporter, ReporterExt};
use crate::results::TestResults;

use rand::rngs::SmallRng;

pub async fn run_all_tests<C: Connector, R: Reporter>(
    configuration: &TestConfiguration,
    connector: &C,
    reporter: &R,
    rng: &mut SmallRng,
    results: &RefCell<TestResults>,
) -> Option<()> {
    let capabilities = reporter
        .nest(
            "Capabilities",
            results,
            capabilities::test_capabilities(configuration, connector, reporter, results),
        )
        .await?;

    let schema = reporter
        .nest(
            "Schema",
            results,
            schema::test_schema(configuration, connector, reporter, results),
        )
        .await?;

    reporter
        .nest(
            "Query",
            results,
            query::test_query(
                configuration,
                connector,
                reporter,
                &capabilities,
                &schema,
                rng,
                results,
            ),
        )
        .await;

    Some(())
}