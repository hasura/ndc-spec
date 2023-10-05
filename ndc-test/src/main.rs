use std::{process::exit, path::PathBuf};

use clap::{Parser, Subcommand};
use ndc_client::apis::configuration::Configuration;
use ndc_test::{TestConfiguration, report};

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
}

#[tokio::main]
async fn main() {
    match Options::parse().command {
        Commands::Test { endpoint, seed, snapshots_dir } => {
            let test_configuration = TestConfiguration { seed, snapshots_dir };

            let configuration = Configuration {
                base_path: endpoint,
                user_agent: None,
                client: reqwest::Client::new(),
                basic_auth: None,
                oauth_access_token: None,
                bearer_access_token: None,
                api_key: None,
            };

            let results = ndc_test::test_connector(&test_configuration, &configuration).await;

            if !results.failures.is_empty() {
                println!();
                println!("{}", report(results));

                exit(1)
            }
        }
    }
}
