use clap::Parser;
use ndc_client::apis::configuration::Configuration;
use ndc_client::apis::default_api as api;

use ndc_test::*;

#[derive(Parser)]
struct Options {
    #[arg(long, value_name = "ENDPOINT")]
    endpoint: String,
}

#[tokio::main]
async fn main() {
    let options = Options::parse();

    let http_client = reqwest::Client::new();

    let configuration = Configuration {
        base_path: options.endpoint,
        user_agent: None,
        client: http_client.clone(),
        basic_auth: None,
        oauth_access_token: None,
        bearer_access_token: None,
        api_key: None,
    };

    println!("Fetching /capabilities");
    let capabilities = api::capabilities_get(&configuration).await.unwrap();

    println!("Validating capabilities");
    validate_capabilities(&capabilities);

    print!("Fetching /schema");
    let schema = api::schema_get(&configuration).await.unwrap();

    println!("Validating schema");
    validate_schema(&schema);

    println!("Testing /query");
    test_query(&configuration, &capabilities, &schema).await;
}
