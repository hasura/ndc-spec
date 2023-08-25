use clap::{Parser, Subcommand};
use ndc_client::apis::configuration::Configuration;
use ndc_test::TestFailure;

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
async fn main() -> Result<(), TestFailure> {
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

            ndc_test::test_connector(&configuration).await
        }
    }
}
