pub mod configuration;
pub mod connector;
pub mod error;
pub mod reporter;
pub mod snapshot;
pub mod test_cases;

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
use reporter::Reporter;
use serde::de::DeserializeOwned;
use snapshot::{snapshot_test, SnapshottingConnector};

#[async_trait(?Send)]
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

    async fn mutation(&self, request: models::MutationRequest) -> Result<models::MutationResponse> {
        Ok(api::mutation_post(self, request).await?)
    }
}

pub async fn test_connector<C: Connector, R: Reporter>(
    configuration: &configuration::TestConfiguration,
    connector: &C,
    reporter: &mut R,
) {
    let mut rng = match configuration.seed {
        None => rand::rngs::SmallRng::from_entropy(),
        Some(seed) => rand::rngs::SmallRng::from_seed(seed),
    };

    let _ = match &configuration.snapshots_dir {
        None => test_cases::run_all_tests(connector, reporter, &mut rng).await,
        Some(snapshot_path) => {
            test_cases::run_all_tests(
                &SnapshottingConnector {
                    snapshot_path,
                    connector,
                },
                reporter,
                &mut rng,
            )
            .await
        }
    };
}

pub async fn test_snapshots_in_directory<C: Connector, R: Reporter>(
    connector: &C,
    reporter: &mut R,
    snapshots_dir: PathBuf,
) {
    let _ = async {
        nest!("Query", reporter, {
            test_snapshots_in_directory_with::<C, R, _, _, _>(
                reporter,
                snapshots_dir.join("query"),
                |req| connector.query(req),
            )
        });

        nest!("Mutation", reporter, {
            Box::pin({
                test_snapshots_in_directory_with::<C, R, _, _, _>(
                    reporter,
                    snapshots_dir.join("mutation"),
                    |req| connector.mutation(req),
                )
            })
        });

        Some(())
    }
    .await;
}

pub async fn test_snapshots_in_directory_with<
    C: Connector,
    R: Reporter,
    Req: DeserializeOwned,
    Res: DeserializeOwned + serde::Serialize + PartialEq,
    F: Future<Output = Result<Res>>,
>(
    reporter: &mut R,
    snapshots_dir: PathBuf,
    f: impl Fn(Req) -> F,
) {
    match std::fs::read_dir(snapshots_dir) {
        Ok(dir) => {
            for entry in dir {
                let entry = entry.expect("Error reading snapshot directory entry");

                test!(
                    entry.file_name().to_str().unwrap_or("{unknown}"),
                    reporter,
                    {
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
                        }
                    }
                );
            }
        }
        Err(e) => println!("Warning: a snapshot folder could not be found: {}", e),
    }
}
