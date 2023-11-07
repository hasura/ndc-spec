use std::{path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use ndc_client::apis::configuration::Configuration;
use ndc_test::{report, TestConfiguration};

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
        endpoint: String,
        #[arg(long, value_name = "SEED")]
        seed: Option<String>,
        #[arg(long, value_name = "PATH")]
        snapshots_dir: Option<PathBuf>,
    },
    Replay {
        #[arg(long, value_name = "ENDPOINT")]
        endpoint: String,
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
        } => {
            let test_configuration = TestConfiguration {
                seed,
                snapshots_dir,
            };

            let configuration = Configuration {
                base_path: endpoint,
                user_agent: None,
                client: reqwest::Client::new(),
            };

            let results = ndc_test::test_connector(&test_configuration, &configuration).await;

            if !results.failures.is_empty() {
                println!();
                println!("{}", report(results));

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
            };

            let results = ndc_test::test_snapshots_in_directory(&configuration, snapshots_dir).await;

            if !results.failures.is_empty() {
                println!();
                println!("{}", report(results));

                exit(1)
            }
        }
    }
}
