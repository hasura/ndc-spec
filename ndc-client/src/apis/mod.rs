use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ConnectorError {
    pub status: reqwest::StatusCode,
    pub error_response: crate::models::ErrorResponse,
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
    ResponseTooLarge(usize),
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
            Error::ResponseTooLarge(limit) => {
                ("response", format!("too large (limit: {} bytes)", limit))
            }
        };
        write!(f, "error in {}: {}", module, e)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Reqwest(e) => Some(e),
            Error::Serde(e) => Some(e),
            Error::Io(e) => Some(e),
            Error::ConnectorError(_)
            | Error::InvalidConnectorError(_)
            | Error::InvalidBaseURL
            | Error::ResponseTooLarge(_) => None,
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

pub fn urlencode<T: AsRef<str>>(s: T) -> String {
    ::url::form_urlencoded::byte_serialize(s.as_ref().as_bytes()).collect()
}

pub mod default_api;

pub mod configuration;
