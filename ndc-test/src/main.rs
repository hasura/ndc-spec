use std::{path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use ndc_test::{
    benchmark_report,
    client::Configuration,
    configuration::{TestConfiguration, TestGenerationConfiguration, TestOptions},
    reporter::{ConsoleReporter, TestResults},
    ReportConfiguration,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Options {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Test {
        #[arg(long, value_name = "ENDPOINT", help = "The NDC endpoint to test")]
        endpoint: reqwest::Url,
        #[arg(
            long,
            value_name = "SEED",
            help = "a 32-byte string with which to initialize the RNG"
        )]
        seed: Option<String>,
        #[arg(
            long,
            value_name = "PATH",
            help = "the directory used to store snapshot files"
        )]
        snapshots_dir: Option<PathBuf>,
        #[arg(
            long,
            value_name = "COUNT",
            default_value = "10",
            help = "the number of test cases to generate per scenario"
        )]
        test_cases: u32,
        #[arg(
            long,
            value_name = "COUNT",
            default_value = "10",
            help = "the number of example rows to fetch from each collection"
        )]
        sample_size: u32,
        #[arg(
            long,
            value_name = "COUNT",
            default_value = "10",
            help = "the maximum number of rows to fetch per test query"
        )]
        max_limit: u32,
        #[arg(
            short='x',
            action = clap::ArgAction::Count,
            help = "increase complexity of generated queries",
        )]
        complexity: u8,
        #[arg(long, help = "Turn off validations for query responses")]
        no_validate_responses: bool,
    },
    Replay {
        #[arg(long, value_name = "ENDPOINT", help = "The NDC endpoint to test")]
        endpoint: reqwest::Url,
        #[arg(
            long,
            value_name = "PATH",
            help = "the directory used to store snapshot files"
        )]
        snapshots_dir: PathBuf,
        #[arg(long, help = "Turn off validations for query responses")]
        no_validate_responses: bool,
    },
    Bench {
        #[arg(long, value_name = "ENDPOINT", help = "The NDC endpoint to test")]
        endpoint: reqwest::Url,
        #[arg(
            long,
            value_name = "PATH",
            help = "the directory used to store snapshot files"
        )]
        snapshots_dir: PathBuf,
        #[arg(
            long,
            value_name = "COUNT",
            help = "the number of samples to collect per test",
            default_value = "100"
        )]
        samples: u32,
        #[arg(
            long,
            value_name = "TOLERANCE",
            help = "tolerable deviation from previous report, in standard deviations from the mean"
        )]
        tolerance: Option<f64>,
    },
}

#[tokio::main]
async fn main() {
    match Options::parse().command {
        Commands::Test {
            endpoint,
            seed,
            snapshots_dir,
            test_cases,
            sample_size,
            max_limit,
            complexity,
            no_validate_responses,
        } => {
            let seed: Option<[u8; 32]> = seed.map(|seed| seed.as_bytes().try_into().unwrap());

            let gen_config = TestGenerationConfiguration {
                test_cases,
                sample_size,
                max_limit,
                complexity,
            };

            let options = TestOptions {
                validate_responses: !no_validate_responses,
            };

            let test_configuration = TestConfiguration {
                seed,
                snapshots_dir,
                options,
                gen_config,
            };

            let configuration = Configuration {
                base_path: endpoint,
                client: reqwest::Client::new(),
            };

            let mut reporter = (ConsoleReporter::default(), TestResults::default());

            ndc_test::test_connector(&test_configuration, &configuration, &mut reporter).await;

            if !reporter.1.failures.is_empty() {
                println!();
                println!("{}", reporter.1.report());

                exit(1)
            }
        }
        Commands::Replay {
            endpoint,
            snapshots_dir,
            no_validate_responses,
        } => {
            let configuration = Configuration {
                base_path: endpoint,
                client: reqwest::Client::new(),
            };

            let mut reporter = (ConsoleReporter::default(), TestResults::default());

            let options = TestOptions {
                validate_responses: !no_validate_responses,
            };

            ndc_test::test_snapshots_in_directory(
                &options,
                &configuration,
                &mut reporter,
                snapshots_dir,
            )
            .await;

            if !reporter.1.failures.is_empty() {
                println!();
                println!("{}", reporter.1.report());

                exit(1)
            }
        }
        Commands::Bench {
            endpoint,
            snapshots_dir,
            samples,
            tolerance,
        } => {
            let configuration = Configuration {
                base_path: endpoint,
                client: reqwest::Client::new(),
            };

            let mut reporter = (ConsoleReporter::default(), TestResults::default());

            let report_configuration = ReportConfiguration { samples, tolerance };

            let report = ndc_test::bench_snapshots_in_directory(
                &report_configuration,
                &configuration,
                &mut reporter,
                snapshots_dir,
            )
            .await
            .unwrap();

            println!();
            println!("{}", benchmark_report(&report_configuration, report));

            if !reporter.1.failures.is_empty() {
                exit(1);
            }
        }
    }
}
