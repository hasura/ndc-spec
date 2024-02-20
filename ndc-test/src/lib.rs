pub mod configuration;
pub mod connector;
pub mod error;
pub mod reporter;
pub mod results;
pub mod snapshot;
pub mod test_cases;

use std::cell::RefCell;

use std::fs::File;
use std::future::Future;

use std::path::PathBuf;

use async_trait::async_trait;
use connector::Connector;
use error::Error;

use ndc_client::apis::configuration::Configuration;
use ndc_client::apis::default_api as api;
use ndc_client::models::{self};

use error::Result;

use rand::SeedableRng;
use reporter::{Reporter, ReporterExt};
use results::TestResults;
use serde::de::DeserializeOwned;
use snapshot::snapshot_test;

#[async_trait]
impl Connector for Configuration {
    async fn get_capabilities(&self) -> Result<models::CapabilitiesResponse> {
        Ok(api::capabilities_get(self).await?)
    }

    async fn get_schema(&self) -> Result<models::SchemaResponse> {
        Ok(api::schema_get(self).await?)
    }

    async fn query(&self, request: models::QueryRequest) -> Result<models::QueryResponse> {
        Ok(api::query_post(self, request).await?)
    }

    async fn mutation(
        &self,
        request: models::MutationRequest,
    ) -> Result<models::MutationResponse> {
        Ok(api::mutation_post(self, request).await?)
    }
}

pub fn report(results: TestResults) -> String {
    use colored::Colorize;

    let mut result = format!("Failed with {0} test failures:", results.failures.len())
        .red()
        .to_string();

    let mut ix = 1;
    for failure in results.failures {
        result += format!("\n\n[{0}] {1}", ix, failure.name).as_str();
        for path_element in failure.path {
            result += format!("\n  in {0}", path_element).as_str();
        }
        result += format!("\nDetails: {0}", failure.error).as_str();
        ix += 1;
    }

    result
}

pub async fn test_connector<C: Connector, R: Reporter>(
    configuration: &configuration::TestConfiguration,
    connector: &C,
    reporter: &R,
) -> TestResults {
    let results = RefCell::new(TestResults {
        path: vec![],
        failures: vec![],
    });

    let mut rng = match configuration.seed {
        None => rand::rngs::SmallRng::from_entropy(),
        Some(seed) => rand::rngs::SmallRng::from_seed(seed),
    };

    let _ = test_cases::run_all_tests(configuration, connector, reporter, &mut rng, &results).await;

    results.into_inner()
}

pub async fn test_snapshots_in_directory<C: Connector, R: Reporter>(
    connector: &C,
    reporter: &R,
    snapshots_dir: PathBuf,
) -> TestResults {
    let results = RefCell::new(TestResults {
        path: vec![],
        failures: vec![],
    });

    let _ = async {
        reporter
            .nest(
                "Query",
                &results,
                test_snapshots_in_directory_with::<C, R, _, _, _>(
                    reporter,
                    snapshots_dir.join("query"),
                    &results,
                    |req| connector.query(req),
                ),
            )
            .await;

        reporter
            .nest(
                "Mutation",
                &results,
                test_snapshots_in_directory_with::<C, R, _, _, _>(
                    reporter,
                    snapshots_dir.join("mutation"),
                    &results,
                    |req| connector.mutation(req),
                ),
            )
            .await;

        Some(())
    }
    .await;

    results.into_inner()
}

pub async fn test_snapshots_in_directory_with<
    C: Connector,
    R: Reporter,
    Req: DeserializeOwned,
    Res: DeserializeOwned + serde::Serialize + PartialEq,
    F: Future<Output = Result<Res>>,
>(
    reporter: &R,
    snapshots_dir: PathBuf,
    results: &RefCell<TestResults>,
    f: impl Fn(Req) -> F,
) {
    match std::fs::read_dir(snapshots_dir) {
        Ok(dir) => {
            for entry in dir {
                let entry = entry.expect("Error reading snapshot directory entry");

                reporter
                    .test(
                        entry.file_name().to_str().unwrap_or("{unknown}"),
                        results,
                        async {
                            let path = entry.path();

                            let snapshot_pathbuf = path.to_path_buf().join("expected.json");
                            let snapshot_path = snapshot_pathbuf.as_path();

                            let request_file = File::open(path.join("request.json"))
                                .map_err(Error::CannotOpenSnapshotFile)?;
                            let request =
                                serde_json::from_reader(request_file).map_err(Error::SerdeError)?;

                            let response = f(request).await?;

                            snapshot_test(snapshot_path, &response)
                        },
                    )
                    .await;
            }
        }
        Err(e) => println!("Warning: a snapshot folder could not be found: {}", e),
    }
}
