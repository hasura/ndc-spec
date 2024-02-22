use std::{path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use ndc_client::apis::configuration::Configuration;
use ndc_test::{
    configuration::TestConfiguration,
    report,
    reporter::{CompositeReporter, ConsoleReporter, TestResults},
};
use reqwest::header::HeaderMap;

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
        #[arg(long, value_name = "ENDPOINT")]
        endpoint: reqwest::Url,
        #[arg(long, value_name = "SEED")]
        seed: Option<String>,
        #[arg(long, value_name = "PATH")]
        snapshots_dir: Option<PathBuf>,
        #[arg(long, value_name = "COUNT", default_value = "10")]
        sample_rows: u32,
    },
    Replay {
        #[arg(long, value_name = "ENDPOINT")]
        endpoint: reqwest::Url,
        #[arg(long, value_name = "PATH")]
        snapshots_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    match Options::parse().command {
        Commands::Test {
            endpoint,
            seed,
            snapshots_dir,
            sample_rows: _,
        } => {
            let seed: Option<[u8; 32]> = seed.map(|seed| seed.as_bytes().try_into().unwrap());

            let test_configuration = TestConfiguration {
                seed,
                snapshots_dir,
            };

            let configuration = Configuration {
                base_path: endpoint,
                user_agent: None,
                client: reqwest::Client::new(),
                headers: HeaderMap::new(),
            };

            let mut reporter =
                CompositeReporter(ConsoleReporter::default(), TestResults::default());

            ndc_test::test_connector(&test_configuration, &configuration, &mut reporter).await;

            let results = reporter.1;

            if !results.failures.is_empty() {
                println!();
                println!("{}", report(&results));

                exit(1)
            }
        }
        Commands::Replay {
            endpoint,
            snapshots_dir,
        } => {
            let configuration = Configuration {
                base_path: endpoint,
                user_agent: None,
                client: reqwest::Client::new(),
                headers: HeaderMap::new(),
            };

            let mut reporter = ConsoleReporter::new();

            ndc_test::test_snapshots_in_directory(&configuration, &mut reporter, snapshots_dir)
                .await;

            // let results = reporter.results();

            // if !results.failures.is_empty() {
            //     println!();
            //     println!("{}", report(results));

            //     exit(1)
            // }
        }
    }
}
