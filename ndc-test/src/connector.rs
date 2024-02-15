use async_trait::async_trait;
use ndc_client::models;

use crate::error::Error;

#[async_trait]
pub trait Connector {
    async fn get_capabilities(&self) -> Result<models::CapabilitiesResponse, Error>;

    async fn get_schema(&self) -> Result<models::SchemaResponse, Error>;

    async fn query(&self, request: models::QueryRequest) -> Result<models::QueryResponse, Error>;

    async fn mutation(
        &self,
        request: models::MutationRequest,
    ) -> Result<models::MutationResponse, Error>;
}
