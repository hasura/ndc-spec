use std::{path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use ndc_client::apis::configuration::Configuration;
use ndc_test::{
    configuration::TestConfiguration,
    reporter::{ConsoleReporter, TestResults},
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
                (ConsoleReporter::default(), TestResults::default());

            ndc_test::test_connector(&test_configuration, &configuration, &mut reporter).await;

            if !reporter.1.failures.is_empty() {
                println!();
                println!("{}", report(&reporter.1));

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

            let mut reporter =
                (ConsoleReporter::default(), TestResults::default());

            ndc_test::test_snapshots_in_directory(&configuration, &mut reporter, snapshots_dir)
                .await;

            if !reporter.1.failures.is_empty() {
                println!();
                println!("{}", report(&reporter.1));

                exit(1)
            }
        }
    }
}

pub fn report(results: &TestResults) -> String {
    use colored::Colorize;

    let mut result = format!("Failed with {0} test failures:", results.failures.len())
        .red()
        .to_string();

    let mut ix = 1;
    for failure in results.failures.iter() {
        result += format!("\n\n[{0}] {1}", ix, failure.name).as_str();
        for path_element in failure.path.iter() {
            result += format!("\n  in {0}", path_element).as_str();
        }
        result += format!("\nDetails: {0}", failure.error).as_str();
        ix += 1;
    }

    result
}
