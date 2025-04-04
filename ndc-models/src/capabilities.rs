use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::RelationalQueryCapabilities;

// ANCHOR: CapabilitiesResponse
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Capabilities Response")]
pub struct CapabilitiesResponse {
    pub version: String,
    pub capabilities: Capabilities,
}
// ANCHOR_END: CapabilitiesResponse

// ANCHOR: LeafCapability
/// A unit value to indicate a particular leaf capability is supported.
/// This is an empty struct to allow for future sub-capabilities.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LeafCapability {}
// ANCHOR_END: LeafCapability

// ANCHOR: Capabilities
/// Describes the features of the specification which a data connector implements.
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Capabilities")]
pub struct Capabilities {
    pub query: QueryCapabilities,
    pub mutation: MutationCapabilities,
    pub relationships: Option<RelationshipCapabilities>,
    /// Does the connector support the relational query API? This feature is experimental and subject
    /// to breaking changes within minor versions.
    pub relational_query: Option<RelationalQueryCapabilities>,
}
// ANCHOR_END: Capabilities

// ANCHOR: QueryCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Query Capabilities")]
pub struct QueryCapabilities {
    /// Does the connector support aggregate queries
    pub aggregates: Option<AggregateCapabilities>,
    /// Does the connector support queries which use variables
    pub variables: Option<LeafCapability>,
    /// Does the connector support explaining queries
    pub explain: Option<LeafCapability>,
    /// Does the connector support nested fields
    #[serde(default)]
    pub nested_fields: NestedFieldCapabilities,
    /// Does the connector support EXISTS predicates
    #[serde(default)]
    pub exists: ExistsCapabilities,
}
// ANCHOR_END: QueryCapabilities

// ANCHOR: ExistsCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Exists Capabilities")]
pub struct ExistsCapabilities {
    /// Does the connector support named scopes in column references inside
    /// EXISTS predicates
    pub named_scopes: Option<LeafCapability>,
    /// Does the connector support ExistsInCollection::Unrelated
    pub unrelated: Option<LeafCapability>,
    /// Does the connector support ExistsInCollection::NestedCollection
    pub nested_collections: Option<LeafCapability>,
    /// Does the connector support filtering over nested scalar arrays using existential quantification.
    /// This means the connector must support ExistsInCollection::NestedScalarCollection.
    pub nested_scalar_collections: Option<LeafCapability>,
}
// ANCHOR_END: ExistsCapabilities

// ANCHOR: NestedFieldCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Nested Field Capabilities")]
pub struct NestedFieldCapabilities {
    /// Does the connector support filtering by values of nested fields
    pub filter_by: Option<NestedFieldFilterByCapabilities>,
    /// Does the connector support ordering by values of nested fields
    pub order_by: Option<LeafCapability>,
    /// Does the connector support aggregating values within nested fields
    pub aggregates: Option<LeafCapability>,
    /// Does the connector support nested collection queries using
    /// `NestedField::NestedCollection`
    pub nested_collections: Option<LeafCapability>,
}
// ANCHOR_END: NestedFieldCapabilities

// ANCHOR: NestedFieldFilterByCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Nested Field Filter By Capabilities")]
pub struct NestedFieldFilterByCapabilities {
    /// Does the connector support filtering over nested arrays (ie. Expression::ArrayComparison)
    pub nested_arrays: Option<NestedArrayFilterByCapabilities>,
}
// ANCHOR_END: NestedFieldFilterByCapabilities

// ANCHOR: NestedArrayFilterByCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Nested Array Filter By Capabilities")]
pub struct NestedArrayFilterByCapabilities {
    /// Does the connector support filtering over nested arrays by checking if the array contains a value.
    /// This must be supported for all types that can be contained in an array that implement an 'eq'
    /// comparison operator.
    pub contains: Option<LeafCapability>,
    /// Does the connector support filtering over nested arrays by checking if the array is empty.
    /// This must be supported no matter what type is contained in the array.
    pub is_empty: Option<LeafCapability>,
}
// ANCHOR_END: NestedArrayFilterByCapabilities

// ANCHOR: AggregateCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Aggregate Capabilities")]
pub struct AggregateCapabilities {
    /// Does the connector support filtering based on aggregated values
    pub filter_by: Option<LeafCapability>,
    /// Does the connector support aggregations over groups
    pub group_by: Option<GroupByCapabilities>,
}
// ANCHOR_END: AggregateCapabilities

// ANCHOR: GroupByCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Group By Capabilities")]
pub struct GroupByCapabilities {
    /// Does the connector support post-grouping predicates
    pub filter: Option<LeafCapability>,
    /// Does the connector support post-grouping ordering
    pub order: Option<LeafCapability>,
    /// Does the connector support post-grouping pagination
    pub paginate: Option<LeafCapability>,
}
// ANCHOR_END: GroupByCapabilities

// ANCHOR: MutationCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Mutation Capabilities")]
pub struct MutationCapabilities {
    /// Does the connector support executing multiple mutations in a transaction.
    pub transactional: Option<LeafCapability>,
    /// Does the connector support explaining mutations
    pub explain: Option<LeafCapability>,
}
// ANCHOR_END: MutationCapabilities

// ANCHOR: RelationshipCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relationship Capabilities")]
pub struct RelationshipCapabilities {
    /// Does the connector support comparisons that involve related collections (ie. joins)?
    pub relation_comparisons: Option<LeafCapability>,
    /// Does the connector support ordering by an aggregated array relationship?
    pub order_by_aggregate: Option<LeafCapability>,
    /// Does the connector support navigating a relationship from inside a nested object
    pub nested: Option<NestedRelationshipCapabilities>,
}
// ANCHOR_END: RelationshipCapabilities

// ANCHOR: NestedRelationshipCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Nested Relationship Capabilities")]
pub struct NestedRelationshipCapabilities {
    /// Does the connector support navigating a relationship from inside a nested object inside a nested array
    pub array: Option<LeafCapability>,
    /// Does the connector support filtering over a relationship that starts from inside a nested object
    pub filtering: Option<LeafCapability>,
    /// Does the connector support ordering over a relationship that starts from inside a nested object
    pub ordering: Option<LeafCapability>,
}
// ANCHOR_END: NestedRelationshipCapabilities
