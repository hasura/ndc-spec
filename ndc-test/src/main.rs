use clap::Parser;
use ndc_client::apis::configuration::Configuration;

#[derive(Parser)]
struct Options {
    #[arg(long, value_name = "ENDPOINT")]
    endpoint: String,
}

#[tokio::main]
async fn main() -> Result<(), ndc_client::apis::Error> {
    let options = Options::parse();
    let configuration = Configuration {
        base_path: options.endpoint,
        user_agent: None,
        client: reqwest::Client::new(),
        basic_auth: None,
        oauth_access_token: None,
        bearer_access_token: None,
        api_key: None,
    };

    ndc_test::test_connector(&configuration).await
}
