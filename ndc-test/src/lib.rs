#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_precision_loss)]

pub mod client;
pub mod configuration;
pub mod connector;
pub mod error;
pub mod reporter;
pub mod snapshot;
pub mod test_cases;

use std::collections::BTreeMap;
use std::fs::File;
use std::future::Future;

use std::path::PathBuf;
use std::time::Instant;

use async_trait::async_trait;
use colorful::Colorful;
use connector::Connector;
use error::Error;

use ndc_models as models;

use error::Result;

use rand::SeedableRng;
use reporter::Reporter;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use snapshot::{snapshot_test, SnapshottingConnector};

use crate::test_cases::query::validate::ValidatingConnector;

#[async_trait(?Send)]
impl Connector for client::Configuration {
    async fn get_capabilities(&self) -> Result<models::CapabilitiesResponse> {
        Ok(client::capabilities_get(self).await?)
    }

    async fn get_schema(&self) -> Result<models::SchemaResponse> {
        Ok(client::schema_get(self).await?)
    }

    async fn query(&self, request: models::QueryRequest) -> Result<models::QueryResponse> {
        Ok(client::query_post(self, request).await?)
    }

    async fn mutation(&self, request: models::MutationRequest) -> Result<models::MutationResponse> {
        Ok(client::mutation_post(self, request).await?)
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
        None => {
            test_cases::run_all_tests(
                &configuration.gen_config,
                &configuration.options,
                connector,
                reporter,
                &mut rng,
            )
            .await
        }
        Some(snapshot_path) => {
            test_cases::run_all_tests(
                &configuration.gen_config,
                &configuration.options,
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
    options: &configuration::TestOptions,
    connector: &C,
    reporter: &mut R,
    snapshots_dir: PathBuf,
) -> Option<()> {
    let schema = test!("Schema", reporter, connector.get_schema())?;

    nest!("Query", reporter, async {
        if options.validate_responses {
            let connector = ValidatingConnector {
                connector,
                schema: &schema,
            };
            test_snapshots_in_directory_with::<C, R, _, _, _>(
                reporter,
                snapshots_dir.join("query"),
                |req| connector.query(req),
            )
            .await;
        } else {
            test_snapshots_in_directory_with::<C, R, _, _, _>(
                reporter,
                snapshots_dir.join("query"),
                |req| connector.query(req),
            )
            .await;
        }
    });

    nest!("Mutation", reporter, {
        test_snapshots_in_directory_with::<C, R, _, _, _>(
            reporter,
            snapshots_dir.join("mutation"),
            |req| connector.mutation(req),
        )
    });

    Some(())
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
    if snapshots_dir.exists() {
        let entries =
            read_dir_sorted_by_name(snapshots_dir).expect("Unable to read snapshot directory");
        for entry in entries {
            if entry.metadata().is_ok_and(|md| md.is_dir()) {
                test!(
                    entry.file_name().to_str().unwrap_or("{unknown}"),
                    reporter,
                    {
                        async {
                            let path = entry.path();

                            let snapshot_pathbuf = path.join("expected.json");
                            let snapshot_path = snapshot_pathbuf.as_path();

                            let request_file = File::open(path.join("request.json"))
                                .map_err(Error::CannotOpenSnapshotFile)?;
                            let request = serde_json::from_reader(request_file)?;

                            let response = f(request).await?;

                            snapshot_test(snapshot_path, &response)
                        }
                    }
                );
            }
        }
    } else {
        println!("Warning: a snapshot folder could not be found: {snapshots_dir:?}");
    }
}

// Read directory and sort inner files by alphabet
fn read_dir_sorted_by_name(snapshots_dir: PathBuf) -> std::io::Result<Vec<std::fs::DirEntry>> {
    let mut paths: Vec<_> =
        std::fs::read_dir(snapshots_dir)?.collect::<std::io::Result<Vec<_>>>()?;
    paths.sort_by_key(std::fs::DirEntry::path);

    Ok(paths)
}

#[derive(Debug, Clone)]
pub struct ReportConfiguration {
    pub samples: u32,
    pub tolerance: Option<f64>,
}

pub async fn bench_snapshots_in_directory<C: Connector + Send, R: Reporter>(
    config: &ReportConfiguration,
    connector: &C,
    reporter: &mut R,
    snapshots_dir: PathBuf,
) -> Result<BTreeMap<String, Statistics>> {
    nest!("Query", reporter, {
        bench_snapshots_in_directory_with::<R, _, _, _>(
            config,
            reporter,
            snapshots_dir.join("query"),
            |req| connector.query(req),
        )
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub samples: u32,
    pub mean: f64,
    pub mean_delta: Option<f64>,
    pub mean_deviation: Option<f64>,
    pub sample_variance: f64,
    pub min: f64,
    pub max: f64,
}

pub async fn bench_snapshots_in_directory_with<
    R: Reporter,
    Req: Clone + DeserializeOwned,
    Res,
    F: Future<Output = Result<Res>>,
>(
    config: &ReportConfiguration,
    reporter: &mut R,
    snapshots_dir: PathBuf,
    f: impl Fn(Req) -> F,
) -> Result<BTreeMap<String, Statistics>> {
    match std::fs::read_dir(snapshots_dir) {
        Ok(dir) => {
            let mut reports = BTreeMap::new();

            for entry in dir {
                let entry = entry.expect("Error reading snapshot directory entry");

                test!(
                    entry.file_name().to_str().unwrap_or("{unknown}"),
                    reporter,
                    {
                        async {
                            let path = entry.path();

                            let report_path = path.join("report.json");

                            let prev_report: Option<Statistics> = if report_path.exists() {
                                let prev_report_file = File::open(report_path.clone())
                                    .map_err(Error::CannotOpenBenchmarkReport)?;
                                Some(serde_json::from_reader(prev_report_file)?)
                            } else {
                                None
                            };

                            let request_file = File::open(path.join("request.json"))
                                .map_err(Error::CannotOpenSnapshotFile)?;
                            let request: Req = serde_json::from_reader(request_file)?;

                            let mut min_d: f64 = f64::MAX;
                            let mut max_d: f64 = f64::MIN;

                            let mut sum_d: f64 = 0.0;
                            let mut sum_d_2: f64 = 0.0;

                            for _iteration in 0..config.samples {
                                let request_copy = request.clone();
                                let start = Instant::now();
                                f(request_copy).await?;
                                let end = Instant::now();

                                let duration = (end - start).as_micros() as f64;

                                min_d = min_d.min(duration);
                                max_d = max_d.max(duration);

                                sum_d += duration;

                                let duration_2 = duration * duration;
                                sum_d_2 += duration_2;
                            }

                            let mean = sum_d / config.samples as f64;
                            let sample_variance = (config.samples as f64 * sum_d_2 - sum_d * sum_d)
                                / (config.samples as f64 - 1.0);

                            let (mean_delta, mean_deviation) = match prev_report {
                                Some(prev_report) => {
                                    let mean_delta = mean - prev_report.mean;
                                    let mean_deviation =
                                        mean_delta / prev_report.sample_variance.sqrt();
                                    (Some(mean_delta), Some(mean_deviation))
                                }
                                None => (None, None),
                            };

                            let report = Statistics {
                                samples: config.samples,
                                mean,
                                mean_delta,
                                mean_deviation,
                                sample_variance,
                                min: min_d,
                                max: max_d,
                            };

                            if let Some(name) = entry.file_name().to_str() {
                                reports.insert(name.into(), report.clone());
                            }

                            if let Some(deviation) = mean_deviation {
                                if let Some(tolerance) = config.tolerance {
                                    if deviation > tolerance {
                                        return Err(Error::BenchmarkExceededTolerance(
                                            deviation.abs(),
                                        ));
                                    }
                                }
                            }

                            let report_json = serde_json::to_string_pretty(&report)?;

                            std::fs::write(report_path, report_json)
                                .map_err(Error::CannotOpenSnapshotFile)?;

                            Ok(())
                        }
                    }
                );
            }

            Ok(reports)
        }
        Err(e) => Err(Error::CannotOpenBenchmarkDirectory(e)),
    }
}

pub fn benchmark_report(
    config: &ReportConfiguration,
    reports: BTreeMap<String, Statistics>,
) -> String {
    if let Some(max_width) = reports.keys().map(String::len).max() {
        let spaces = " ".repeat(max_width + 1);
        let mut result = format!("{spaces}        μ           Δ         σ       min       max\n");

        for (report_name, statistics) in reports {
            let spaces = " ".repeat(max_width + 1 - report_name.len());
            let Statistics {
                samples: _,
                mean,
                mean_delta,
                mean_deviation,
                sample_variance,
                min,
                max,
            } = statistics;

            let delta = mean_delta.map_or("            ".into(), |d| format!(" ({d: >+7.02}μs)"));

            let std_dev = sample_variance.sqrt();
            let line = format!("{report_name}{spaces}{mean: >7.02}μs{delta} {std_dev: >7.02}μs {min: >7.02}μs {max: >7.02}μs \n");

            let line = match (config.tolerance, mean_deviation) {
                (Some(tolerance), Some(deviation)) if deviation > tolerance => {
                    line.red().to_string()
                }
                _ => line,
            };

            result += line.as_str();
        }

        result
    } else {
        String::new()
    }
}
