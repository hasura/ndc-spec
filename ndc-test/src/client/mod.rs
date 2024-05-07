use std::error;
use std::fmt;

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ConnectorError {
    pub status: reqwest::StatusCode,
    pub error_response: ndc_models::ErrorResponse,
}

impl fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConnectorError {{ status: {0}, error_response.message: {1} }}",
            self.status, self.error_response.message
        )
    }
}

#[derive(Debug, Clone)]
pub struct InvalidConnectorError {
    pub status: reqwest::StatusCode,
    pub content: serde_json::Value,
}

impl fmt::Display for InvalidConnectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "InvalidConnectorError {{ status: {0}, content: {1} }}",
            self.status, self.content
        )
    }
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    Io(std::io::Error),
    ConnectorError(ConnectorError),
    InvalidConnectorError(InvalidConnectorError),
    InvalidBaseURL,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (module, e) = match self {
            Error::Reqwest(e) => ("reqwest", e.to_string()),
            Error::Serde(e) => ("serde", e.to_string()),
            Error::Io(e) => ("IO", e.to_string()),
            Error::ConnectorError(e) => ("response", format!("status code {}", e.status)),
            Error::InvalidConnectorError(e) => ("response", format!("status code {}", e.status)),
            Error::InvalidBaseURL => ("url", "invalid base URL".into()),
        };
        write!(f, "error in {module}: {e}")
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Reqwest(e) => Some(e),
            Error::Serde(e) => Some(e),
            Error::Io(e) => Some(e),
            Error::ConnectorError(_) | Error::InvalidConnectorError(_) | Error::InvalidBaseURL => {
                None
            }
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

#[derive(Debug, Clone)]
pub struct Configuration {
    pub base_path: reqwest::Url,
    pub client: reqwest::Client,
}

fn append_path(url: &reqwest::Url, path: &[&str]) -> Result<reqwest::Url, ()> {
    let mut url = url.clone();
    url.path_segments_mut()?.pop_if_empty().extend(path);
    Ok(url)
}

pub(crate) async fn capabilities_get(
    configuration: &Configuration,
) -> Result<ndc_models::CapabilitiesResponse, Error> {
    let client = &configuration.client;

    let uri = append_path(&configuration.base_path, &["capabilities"])
        .map_err(|()| Error::InvalidBaseURL)?;
    let req = client.get(uri).build()?;
    let resp = client.execute(req).await?;

    let response_status = resp.status();
    let response_content = resp.json().await?;

    if !response_status.is_client_error() && !response_status.is_server_error() {
        serde_json::from_value(response_content).map_err(Error::from)
    } else {
        Err(construct_error(response_status, response_content))
    }
}

pub(crate) async fn mutation_post(
    configuration: &Configuration,
    mutation_request: ndc_models::MutationRequest,
) -> Result<ndc_models::MutationResponse, Error> {
    let client = &configuration.client;

    let uri =
        append_path(&configuration.base_path, &["mutation"]).map_err(|()| Error::InvalidBaseURL)?;
    let mut req_builder = client.request(reqwest::Method::POST, uri);

    req_builder = req_builder.json(&mutation_request);

    let req = req_builder.build()?;
    let resp = client.execute(req).await?;

    let response_status = resp.status();
    let response_content = resp.json().await?;

    if !response_status.is_client_error() && !response_status.is_server_error() {
        serde_json::from_value(response_content).map_err(Error::from)
    } else {
        Err(construct_error(response_status, response_content))
    }
}

pub(crate) async fn query_post(
    configuration: &Configuration,
    query_request: ndc_models::QueryRequest,
) -> Result<ndc_models::QueryResponse, Error> {
    let client = &configuration.client;

    let uri =
        append_path(&configuration.base_path, &["query"]).map_err(|()| Error::InvalidBaseURL)?;
    let mut req_builder = client.request(reqwest::Method::POST, uri);

    req_builder = req_builder.json(&query_request);

    let req = req_builder.build()?;
    let resp = client.execute(req).await?;

    let response_status = resp.status();
    let response_content = resp.json().await?;

    if !response_status.is_client_error() && !response_status.is_server_error() {
        serde_json::from_value(response_content).map_err(Error::from)
    } else {
        Err(construct_error(response_status, response_content))
    }
}

pub(crate) async fn schema_get(
    configuration: &Configuration,
) -> Result<ndc_models::SchemaResponse, Error> {
    let client = &configuration.client;

    let uri =
        append_path(&configuration.base_path, &["schema"]).map_err(|()| Error::InvalidBaseURL)?;
    let req = client.get(uri).build()?;
    let resp = client.execute(req).await?;

    let response_status = resp.status();
    let response_content = resp.json().await?;

    if !response_status.is_client_error() && !response_status.is_server_error() {
        serde_json::from_value(response_content).map_err(Error::from)
    } else {
        Err(construct_error(response_status, response_content))
    }
}

fn construct_error(
    response_status: reqwest::StatusCode,
    response_content: serde_json::Value,
) -> Error {
    match ndc_models::ErrorResponse::deserialize(&response_content) {
        Ok(error_response) => {
            let connector_error = ConnectorError {
                status: response_status,
                error_response,
            };
            Error::ConnectorError(connector_error)
        }
        // If we can't read the error response, respond as-is.
        Err(_) => Error::InvalidConnectorError(InvalidConnectorError {
            status: response_status,
            content: response_content,
        }),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_append_path() {
        let url = reqwest::Url::parse("http://hasura.io").unwrap();
        let path = "capabilities";
        let result = super::append_path(&url, &[path]).unwrap();
        assert_eq!(result.as_str(), "http://hasura.io/capabilities");
    }

    #[test]
    fn test_append_path_with_trailing_slash() {
        let url = reqwest::Url::parse("http://hasura.io/").unwrap();
        let path = "capabilities";
        let result = super::append_path(&url, &[path]).unwrap();
        assert_eq!(result.as_str(), "http://hasura.io/capabilities");
    }

    #[test]
    fn test_append_path_with_non_empty_path() {
        let url = reqwest::Url::parse("http://hasura.io/ndc").unwrap();
        let path = "capabilities";
        let result = super::append_path(&url, &[path]).unwrap();
        assert_eq!(result.as_str(), "http://hasura.io/ndc/capabilities");
    }

    #[test]
    fn test_append_path_with_non_empty_path_and_trailing_slash() {
        let url = reqwest::Url::parse("http://hasura.io/ndc/").unwrap();
        let path = "capabilities";
        let result = super::append_path(&url, &[path]).unwrap();
        assert_eq!(result.as_str(), "http://hasura.io/ndc/capabilities");
    }

    #[test]
    fn test_append_paths() {
        let url = reqwest::Url::parse("http://hasura.io/ndc/").unwrap();
        let paths = ["query", "explain"];
        let result = super::append_path(&url, &paths).unwrap();
        assert_eq!(result.as_str(), "http://hasura.io/ndc/query/explain");
    }
}
