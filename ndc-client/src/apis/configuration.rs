use reqwest::header::{HeaderMap, HeaderValue};
/// Configuration for the API client
/// Contains all the information necessary to perform requests.
#[derive(Debug, Clone)]
pub struct Configuration<R: super::ResponseHandler> {
    pub base_path: reqwest::Url,
    pub user_agent: Option<String>,
    pub client: reqwest::Client,
    pub headers: HeaderMap<HeaderValue>,
    pub response_handler: R,
}
