use ndc_client::models::QueryRequest;
use schemars::schema_for;

fn schema_file() -> String {
    let schema = schema_for!(QueryRequest);
    serde_json::to_string_pretty(&schema).unwrap()
}

fn main() {
    println!("{}", schema_file());
}