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
use crate::apis::ResponseContent;

trait ToHeaderString {
    fn to_header_string(self: Self) -> String;
}

impl ToHeaderString for HashMap<String, json::Value> {
    fn to_header_string(self: Self) -> String {
        json::to_value(self).map_or("".to_string(), |val| val.to_string())
    }
}

fn inject_trace_context(builder: RequestBuilder) -> RequestBuilder {
    let ctx = Context::current();
    let mut trace_headers = HashMap::new();
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&ctx, &mut trace_headers);
    });
    let mut local_var_req_builder = builder;
    for (key, value) in trace_headers {
        local_var_req_builder = local_var_req_builder.header(key, value);
    }
    local_var_req_builder
}

impl ToHeaderString for &str {
    fn to_header_string(self: Self) -> String {
        self.to_string()
    }
}

pub async fn capabilities_get(
    configuration: &configuration::Configuration,
) -> Result<crate::models::CapabilitiesResponse, Error> {
    let tracer = global::tracer("engine");
    tracer
        .in_span("capabilities_get", |ctx| async {
            let local_var_configuration = configuration;

            let local_var_client = &local_var_configuration.client;

            let local_var_uri_str = format!("{}/capabilities", local_var_configuration.base_path);
            let mut local_var_req_builder =
                local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

            local_var_req_builder = inject_trace_context(local_var_req_builder);

            if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
                local_var_req_builder = local_var_req_builder
                    .header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
            }

            let local_var_req = local_var_req_builder.build()?;
            let local_var_resp = local_var_client
                .execute(local_var_req)
                .with_traced_errors()
                .await?;

            let local_var_status = local_var_resp.status();
            let local_var_content = local_var_resp
                .text()
                .with_traced_errors()
                .with_context(ctx)
                .await?;

            if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
                serde_json::from_str(&local_var_content).map_err(Error::from)
            } else {
                let local_var_entity: Option<serde_json::Value> =
                    serde_json::from_str(&local_var_content).ok();
                let local_var_error = ResponseContent {
                    status: local_var_status,
                    content: local_var_content,
                    entity: local_var_entity,
                };
                Err(Error::ResponseError(local_var_error))
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
            let local_var_configuration = configuration;

            let local_var_client = &local_var_configuration.client;

            let local_var_uri_str = format!("{}/explain", local_var_configuration.base_path);
            let mut local_var_req_builder =
                local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

            if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
                local_var_req_builder = local_var_req_builder
                    .header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
            }
            local_var_req_builder = local_var_req_builder.json(&query_request);

            local_var_req_builder = inject_trace_context(local_var_req_builder);

            let local_var_req = local_var_req_builder.build()?;
            let local_var_resp = local_var_client
                .execute(local_var_req)
                .with_traced_errors()
                .await?;

            let local_var_status = local_var_resp.status();
            let local_var_content = local_var_resp
                .text()
                .with_traced_errors()
                .with_context(ctx)
                .await?;

            if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
                serde_json::from_str(&local_var_content).map_err(Error::from)
            } else {
                let local_var_entity: Option<serde_json::Value> =
                    serde_json::from_str(&local_var_content).ok();
                let local_var_error = ResponseContent {
                    status: local_var_status,
                    content: local_var_content,
                    entity: local_var_entity,
                };
                Err(Error::ResponseError(local_var_error))
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
            let local_var_configuration = configuration;

            let local_var_client = &local_var_configuration.client;

            let local_var_uri_str = format!("{}/mutation", local_var_configuration.base_path);
            let mut local_var_req_builder =
                local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

            if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
                local_var_req_builder = local_var_req_builder
                    .header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
            }
            local_var_req_builder = local_var_req_builder.json(&mutation_request);

            local_var_req_builder = inject_trace_context(local_var_req_builder);

            let local_var_req = local_var_req_builder.build()?;
            let local_var_resp = local_var_client
                .execute(local_var_req)
                .with_traced_errors()
                .await?;

            let local_var_status = local_var_resp.status();
            let local_var_content = local_var_resp
                .text()
                .with_traced_errors()
                .with_context(ctx)
                .await?;

            if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
                serde_json::from_str(&local_var_content).map_err(Error::from)
            } else {
                let local_var_entity: Option<serde_json::Value> =
                    serde_json::from_str(&local_var_content).ok();
                let local_var_error = ResponseContent {
                    status: local_var_status,
                    content: local_var_content,
                    entity: local_var_entity,
                };
                Err(Error::ResponseError(local_var_error))
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
                let local_var_configuration = configuration;

                let local_var_client = &local_var_configuration.client;

                let local_var_uri_str = format!("{}/query", local_var_configuration.base_path);
                let mut local_var_req_builder =
                    local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

                if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
                    local_var_req_builder = local_var_req_builder
                        .header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
                }
                local_var_req_builder = local_var_req_builder.json(&query_request);

                local_var_req_builder = inject_trace_context(local_var_req_builder);

                let local_var_req = local_var_req_builder.build()?;
                let local_var_resp = local_var_client
                    .execute(local_var_req)
                    .with_traced_errors()
                    .await?;

                let local_var_status = local_var_resp.status();
                let local_var_content = local_var_resp.text().with_traced_errors().await?;

                if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
                    serde_json::from_str(&local_var_content).map_err(Error::from)
                } else {
                    let local_var_entity: Option<serde_json::Value> =
                        serde_json::from_str(&local_var_content).ok();
                    let local_var_error = ResponseContent {
                        status: local_var_status,
                        content: local_var_content,
                        entity: local_var_entity,
                    };
                    Err(Error::ResponseError(local_var_error))
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
            let local_var_configuration = configuration;

            let local_var_client = &local_var_configuration.client;

            let local_var_uri_str = format!("{}/schema", local_var_configuration.base_path);
            let mut local_var_req_builder =
                local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

            local_var_req_builder = inject_trace_context(local_var_req_builder);

            if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
                local_var_req_builder = local_var_req_builder
                    .header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
            }

            let local_var_req = local_var_req_builder.build()?;
            let local_var_resp = local_var_client
                .execute(local_var_req)
                .with_traced_errors()
                .await?;

            let local_var_status = local_var_resp.status();
            let local_var_content = local_var_resp
                .text()
                .with_traced_errors()
                .with_context(ctx)
                .await?;

            if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
                serde_json::from_str(&local_var_content).map_err(Error::from)
            } else {
                let local_var_entity: Option<serde_json::Value> =
                    serde_json::from_str(&local_var_content).ok();
                let local_var_error = ResponseContent {
                    status: local_var_status,
                    content: local_var_content,
                    entity: local_var_entity,
                };
                Err(Error::ResponseError(local_var_error))
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
