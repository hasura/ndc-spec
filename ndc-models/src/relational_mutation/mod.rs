use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::LeafCapability;

// ANCHOR: RelationalMutationCapabilities
/// Describes which features of the relational mutation API are supported by the connector.
/// This feature is experimental and subject to breaking changes within minor versions.
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Mutation Capabilities")]
pub struct RelationalMutationCapabilities {
    pub insert: Option<LeafCapability>,
}
// ANCHOR_END: RelationalMutationCapabilities
