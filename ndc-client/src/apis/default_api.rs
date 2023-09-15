use opentelemetry::{
    global,
    trace::{FutureExt, Tracer},
    Context,
};
use reqwest::{self, RequestBuilder};
use serde_json as json;
use std::collections::HashMap;

use self::utils::FutureTracing;

use super::{configuration, Error};

trait ToHeaderString {
    fn to_header_string(self) -> String;
}

impl ToHeaderString for HashMap<String, json::Value> {
    fn to_header_string(self) -> String {
        json::to_value(self).map_or("".to_string(), |val| val.to_string())
    }
}

fn inject_trace_context(builder: RequestBuilder) -> RequestBuilder {
    let ctx = Context::current();
    let mut trace_headers = HashMap::new();
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&ctx, &mut trace_headers);
    });
    let mut req_builder = builder;
    for (key, value) in trace_headers {
        req_builder = req_builder.header(key, value);
    }
    req_builder
}

impl ToHeaderString for &str {
    fn to_header_string(self) -> String {
        self.to_string()
    }
}

pub async fn capabilities_get(
    configuration: &configuration::Configuration,
) -> Result<crate::models::CapabilitiesResponse, Error> {
    let tracer = global::tracer("engine");
    tracer
        .in_span("capabilities_get", |ctx| async {
            let configuration = configuration;

            let client = &configuration.client;

            let uri_str = format!("{}/capabilities", configuration.base_path);
            let mut req_builder =
                client.request(reqwest::Method::GET, uri_str.as_str());

            req_builder = inject_trace_context(req_builder);

            if let Some(ref user_agent) = configuration.user_agent {
                req_builder = req_builder
                    .header(reqwest::header::USER_AGENT, user_agent.clone());
            }

            if let Some(ref bearer_token) = configuration.bearer_access_token {
                req_builder = req_builder.bearer_auth(bearer_token.as_str());
            }

            let req = req_builder.build()?;
            let resp = client
                .execute(req)
                .with_traced_errors()
                .await?;

            let response_status = resp.status();
            let response_content = resp
                .json()
                .with_traced_errors()
                .with_context(ctx)
                .await?;

            if !response_status.is_client_error() && !response_status.is_server_error() {
                serde_json::from_value(response_content).map_err(Error::from)
            } else {
                let error_response: crate::models::ErrorResponse =
                    serde_json::from_value(response_content)?;
                let connector_error = super::ConnectorError {
                    status: response_status,
                    error_response,
                };
                Err(Error::ConnectorError(connector_error))
            }
        })
        .await
}

pub async fn explain_post(
    configuration: &configuration::Configuration,
    query_request: crate::models::QueryRequest,
) -> Result<crate::models::ExplainResponse, Error> {
    let tracer = global::tracer("engine");
    tracer
        .in_span("explain_post", |ctx| async {
            let configuration = configuration;

            let client = &configuration.client;

            let uri_str = format!("{}/explain", configuration.base_path);
            let mut req_builder =
                client.request(reqwest::Method::POST, uri_str.as_str());

            if let Some(ref user_agent) = configuration.user_agent {
                req_builder = req_builder
                    .header(reqwest::header::USER_AGENT, user_agent.clone());
            }

            if let Some(ref bearer_token) = configuration.bearer_access_token {
                req_builder = req_builder.bearer_auth(bearer_token.as_str());
            }

            req_builder = req_builder.json(&query_request);

            req_builder = inject_trace_context(req_builder);

            let req = req_builder.build()?;
            let resp = client
                .execute(req)
                .with_traced_errors()
                .await?;

            let response_status = resp.status();
            let response_content = resp
                .json()
                .with_traced_errors()
                .with_context(ctx)
                .await?;

            if !response_status.is_client_error() && !response_status.is_server_error() {
                serde_json::from_value(response_content).map_err(Error::from)
            } else {
                let error_response: crate::models::ErrorResponse =
                    serde_json::from_value(response_content)?;
                let connector_error = super::ConnectorError {
                    status: response_status,
                    error_response,
                };
                Err(Error::ConnectorError(connector_error))
            }
        })
        .await
}

pub async fn mutation_post(
    configuration: &configuration::Configuration,
    mutation_request: crate::models::MutationRequest,
) -> Result<crate::models::MutationResponse, Error> {
    let tracer = global::tracer("engine");
    tracer
        .in_span("mutation_post", |ctx| async {
            let configuration = configuration;

            let client = &configuration.client;

            let uri_str = format!("{}/mutation", configuration.base_path);
            let mut req_builder =
                client.request(reqwest::Method::POST, uri_str.as_str());

            if let Some(ref user_agent) = configuration.user_agent {
                req_builder = req_builder
                    .header(reqwest::header::USER_AGENT, user_agent.clone());
            }

            if let Some(ref bearer_token) = configuration.bearer_access_token {
                req_builder = req_builder.bearer_auth(bearer_token.as_str());
            }

            req_builder = req_builder.json(&mutation_request);

            req_builder = inject_trace_context(req_builder);

            let req = req_builder.build()?;
            let resp = client
                .execute(req)
                .with_traced_errors()
                .await?;

            let response_status = resp.status();
            let response_content = resp
                .json()
                .with_traced_errors()
                .with_context(ctx)
                .await?;

            if !response_status.is_client_error() && !response_status.is_server_error() {
                serde_json::from_value(response_content).map_err(Error::from)
            } else {
                let error_response: crate::models::ErrorResponse =
                    serde_json::from_value(response_content)?;
                let connector_error = super::ConnectorError {
                    status: response_status,
                    error_response,
                };
                Err(Error::ConnectorError(connector_error))
            }
        })
        .await
}

pub async fn query_post(
    configuration: &configuration::Configuration,
    query_request: crate::models::QueryRequest,
) -> Result<crate::models::QueryResponse, Error> {
    let tracer = global::tracer("engine");
    tracer
        .in_span("query_post", |ctx| {
            async {
                let configuration = configuration;

                let client = &configuration.client;

                let uri_str = format!("{}/query", configuration.base_path);
                let mut req_builder =
                    client.request(reqwest::Method::POST, uri_str.as_str());

                if let Some(ref user_agent) = configuration.user_agent {
                    req_builder = req_builder
                        .header(reqwest::header::USER_AGENT, user_agent.clone());
                }

                if let Some(ref bearer_token) = configuration.bearer_access_token {
                    req_builder = req_builder.bearer_auth(bearer_token.as_str());
                }

                req_builder = req_builder.json(&query_request);

                req_builder = inject_trace_context(req_builder);

                let req = req_builder.build()?;
                let resp = client
                    .execute(req)
                    .with_traced_errors()
                    .await?;

                let response_status = resp.status();
                let response_content = resp.json().with_traced_errors().await?;

                if !response_status.is_client_error() && !response_status.is_server_error() {
                    serde_json::from_value(response_content).map_err(Error::from)
                } else {
                    let error_response: crate::models::ErrorResponse =
                        serde_json::from_value(response_content)?;
                    let connector_error = super::ConnectorError {
                        status: response_status,
                        error_response,
                    };
                    Err(Error::ConnectorError(connector_error))
                }
            }
            .with_context(ctx)
        })
        .await
}

pub async fn schema_get(
    configuration: &configuration::Configuration,
) -> Result<crate::models::SchemaResponse, Error> {
    let tracer = global::tracer("engine");
    tracer
        .in_span("schema_get", |ctx| async {
            let configuration = configuration;

            let client = &configuration.client;

            let uri_str = format!("{}/schema", configuration.base_path);
            let mut req_builder =
                client.request(reqwest::Method::GET, uri_str.as_str());

            req_builder = inject_trace_context(req_builder);

            if let Some(ref user_agent) = configuration.user_agent {
                req_builder = req_builder
                    .header(reqwest::header::USER_AGENT, user_agent.clone());
            }

            if let Some(ref bearer_token) = configuration.bearer_access_token {
                req_builder = req_builder.bearer_auth(bearer_token.as_str());
            }

            let req = req_builder.build()?;
            let resp = client
                .execute(req)
                .with_traced_errors()
                .await?;

            let response_status = resp.status();
            let response_content = resp
                .json()
                .with_traced_errors()
                .with_context(ctx)
                .await?;

            if !response_status.is_client_error() && !response_status.is_server_error() {
                serde_json::from_value(response_content).map_err(Error::from)
            } else {
                let error_response: crate::models::ErrorResponse =
                    serde_json::from_value(response_content)?;
                let connector_error = super::ConnectorError {
                    status: response_status,
                    error_response,
                };
                Err(Error::ConnectorError(connector_error))
            }
        })
        .await
}

mod utils {
    use async_trait::async_trait;
    use opentelemetry::trace::get_active_span;
    use std::{fmt::Display, future::Future};

    pub trait Tracing {
        /// Trace errors to the current trace span
        fn with_traced_errors(self) -> Self;
    }

    impl<A, E: Display> Tracing for Result<A, E> {
        fn with_traced_errors(self) -> Self {
            match self {
                Ok(x) => Ok(x),
                Err(e) => {
                    log_in_current_span(&e);
                    Err(e)
                }
            }
        }
    }

    #[async_trait]
    pub trait FutureTracing: Future {
        /// Trace errors to the current trace span
        async fn with_traced_errors(self) -> Self::Output;
    }

    #[async_trait]
    impl<A, E: Display, T: Future<Output = Result<A, E>> + Send> FutureTracing for T {
        async fn with_traced_errors(self: T) -> Result<A, E> {
            self.await.with_traced_errors()
        }
    }

    pub fn log_in_current_span<E: Display>(e: &E) {
        get_active_span(|span| {
            span.set_status(opentelemetry::trace::Status::Error {
                description: e.to_string().into(),
            });
        });
    }
}
