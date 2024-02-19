use async_trait::async_trait;
use ndc_client::models;

use crate::error::Result;

#[async_trait]
pub trait Connector {
    async fn get_capabilities(&self) -> Result<models::CapabilitiesResponse>;

    async fn get_schema(&self) -> Result<models::SchemaResponse>;

    async fn query(&self, request: models::QueryRequest) -> Result<models::QueryResponse>;

    async fn mutation(
        &self,
        request: models::MutationRequest,
    ) -> Result<models::MutationResponse>;
}
