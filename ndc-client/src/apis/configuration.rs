/// Configuration for the API client
/// Contains all the information necessary to perform requests.
/// 
/// Please note that there is no headers field, headers (for authentication and custom headers) are supposed to be added
/// as default headers in the `reqwest::Client` itself.
/// 
/// # Example
/// 
/// ```rs
/// use ndc_client::apis::configuration::Configuration;
/// 
/// let mut headers = HeaderMap::new();
/// headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer ..."));
/// headers.insert(HeaderName::from_static("x-header"), HeaderValue::from_static("..."));
/// 
/// let client_builder = ClientBuilder::new().default_headers(headers);
/// let client = client_builder.build().unwrap(); // handle errors elegantly in your app
/// 
/// let configuration = Configuration {
///    base_path: "https://api.example.com".to_owned(),
///    user_agent: Some("GraphQL-Engine/3.0.0/rust".to_owned()),
///    client,
///    api_key: Some(SECRET_KEY.to_owned()),
/// };
/// ```
/// 
/// Now this configuration can be used to perform some API requests.
/// 
/// ```rs
/// use ndc_client::apis::configuration::default_api::schema_get;
/// 
/// let schema_response = schema_get(&configuration).await;
/// ```
/// 
#[derive(Debug, Clone)]
pub struct Configuration {
    pub base_path: String,
    pub user_agent: Option<String>,
    pub client: reqwest::Client,
}
