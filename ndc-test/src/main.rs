use std::process::exit;

use clap::{Parser, Subcommand};
use ndc_client::apis::configuration::Configuration;

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
    },
}

#[tokio::main]
async fn main() -> () {
    match Options::parse().command {
        Commands::Test {
            endpoint,
            seed,
        } => {
            let configuration = Configuration {
                base_path: endpoint,
                user_agent: None,
                client: reqwest::Client::new(),
                basic_auth: None,
                oauth_access_token: None,
                bearer_access_token: None,
                api_key: None,
                seed,
            };

            let results = ndc_test::test_connector(&configuration).await;

            if !results.failures.is_empty() {
                println!("\x1b[1;31mFailed with {0} test failures, see stderr for details.\x1b[22;0m", results.failures.len());
                exit(1)
            }
        }
    }
}
