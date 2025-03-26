use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::LeafCapability;

// ANCHOR: RelationalQueryCapabilities
/// Describes which features of the relational query API are supported by the connector.
/// This feature is experimental and subject to breaking changes within minor versions.
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Query Capabilities")]
pub struct RelationalQueryCapabilities {
    pub project: RelationalProjectionCapabilities,
    pub filter: Option<RelationalExpressionCapabilities>,
    pub sort: Option<RelationalSortCapabilities>,
    pub join: Option<RelationalJoinCapabilities>,
    pub aggregate: Option<RelationalAggregateCapabilities>,
    pub window: Option<RelationalWindowCapabilities>,
}
// ANCHOR_END: RelationalQueryCapabilities

// ANCHOR: RelationalProjectionCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Projection Capabilities")]
pub struct RelationalProjectionCapabilities {
    pub expression: RelationalExpressionCapabilities,
}
// ANCHOR_END: RelationalProjectionCapabilities

// ANCHOR: RelationalSortCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Sort Capabilities")]
pub struct RelationalSortCapabilities {
    pub expression: RelationalExpressionCapabilities,
}
// ANCHOR_END: RelationalSortCapabilities

// ANCHOR: RelationalJoinCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Join Capabilities")]
pub struct RelationalJoinCapabilities {
    pub expression: RelationalExpressionCapabilities,
    pub join_types: RelationalJoinTypeCapabilities,
}
// ANCHOR_END: RelationalJoinCapabilities

// ANCHOR: RelationalJoinTypeCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Join Type Capabilities")]
pub struct RelationalJoinTypeCapabilities {
    pub left: Option<LeafCapability>,
    pub right: Option<LeafCapability>,
    pub inner: Option<LeafCapability>,
    pub full: Option<LeafCapability>,
}
// ANCHOR_END: RelationalJoinTypeCapabilities

// ANCHOR: RelationalAggregateCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Aggregate Capabilities")]
pub struct RelationalAggregateCapabilities {
    pub expression: RelationalExpressionCapabilities,
    pub group_by: Option<LeafCapability>,
}
// ANCHOR_END: RelationalAggregateCapabilities

// ANCHOR: RelationalWindowCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Window Capabilities")]
pub struct RelationalWindowCapabilities {
    pub expression: RelationalExpressionCapabilities,
}
// ANCHOR_END: RelationalWindowCapabilities

// ANCHOR: RelationalExpressionCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Expression Capabilities")]
pub struct RelationalExpressionCapabilities {
    pub case: RelationalConditionalExpressionCapabilities,
    pub comparison: RelationalComparisonExpressionCapabilities,
    pub scalar: RelationalScalarExpressionCapabilities,
    pub aggregate: RelationalAggregateExpressionCapabilities,
    pub window: RelationalWindowExpressionCapabilities,
}
// ANCHOR_END: RelationalExpressionCapabilities

// ANCHOR: RelationalConditionalExpressionCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Conditional Expression Capabilities")]
pub struct RelationalConditionalExpressionCapabilities {
    pub case: Option<LeafCapability>,
}
// ANCHOR_END: RelationalConditionalExpressionCapabilities

// ANCHOR: RelationalFilterExpressionCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Filter Expression Capabilities")]
pub struct RelationalComparisonExpressionCapabilities {
    pub like: Option<LeafCapability>,
    pub ilike: Option<LeafCapability>,
    pub between: Option<LeafCapability>,
}
// ANCHOR_END: RelationalFilterExpressionCapabilities

// ANCHOR: RelationalScalarExpressionCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Scalar Expression Capabilities")]
pub struct RelationalScalarExpressionCapabilities {
    pub abs: Option<LeafCapability>,
    pub btrim: Option<LeafCapability>,
    pub ceil: Option<LeafCapability>,
    pub character_length: Option<LeafCapability>,
    pub concat: Option<LeafCapability>,
    pub contains: Option<LeafCapability>,
    pub cos: Option<LeafCapability>,
    pub current_date: Option<LeafCapability>,
    pub current_time: Option<LeafCapability>,
    pub current_timestamp: Option<LeafCapability>,
    pub date_part: Option<LeafCapability>,
    pub date_trunc: Option<LeafCapability>,
    pub exp: Option<LeafCapability>,
    pub floor: Option<LeafCapability>,
    pub greatest: Option<LeafCapability>,
    pub is_nan: Option<LeafCapability>,
    pub is_zero: Option<LeafCapability>,
    pub least: Option<LeafCapability>,
    pub left: Option<LeafCapability>,
    pub ln: Option<LeafCapability>,
    pub log: Option<LeafCapability>,
    pub log10: Option<LeafCapability>,
    pub log2: Option<LeafCapability>,
    pub lpad: Option<LeafCapability>,
    pub ltrim: Option<LeafCapability>,
    pub nvl: Option<LeafCapability>,
    pub power: Option<LeafCapability>,
    pub random: Option<LeafCapability>,
    pub replace: Option<LeafCapability>,
    pub reverse: Option<LeafCapability>,
    pub right: Option<LeafCapability>,
    pub round: Option<LeafCapability>,
    pub rpad: Option<LeafCapability>,
    pub rtrim: Option<LeafCapability>,
    pub sqrt: Option<LeafCapability>,
    pub str_pos: Option<LeafCapability>,
    pub substr: Option<LeafCapability>,
    pub substr_index: Option<LeafCapability>,
    pub tan: Option<LeafCapability>,
    pub to_date: Option<LeafCapability>,
    pub to_timestamp: Option<LeafCapability>,
    pub trunc: Option<LeafCapability>,
    pub to_lower: Option<LeafCapability>,
    pub to_upper: Option<LeafCapability>,
}
// ANCHOR_END: RelationalScalarExpressionCapabilities

// ANCHOR: RelationalAggregateExpressionCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Aggregate Expression Capabilities")]
pub struct RelationalAggregateExpressionCapabilities {
    bool_and: Option<LeafCapability>,
    bool_or: Option<LeafCapability>,
    first_value: Option<LeafCapability>,
    last_value: Option<LeafCapability>,
    mean: Option<LeafCapability>,
    median: Option<LeafCapability>,
    string_agg: Option<LeafCapability>,
    var: Option<LeafCapability>,
}
// ANCHOR_END: RelationalAggregateExpressionCapabilities

// ANCHOR: RelationalWindowExpressionCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Window Expression Capabilities")]
pub struct RelationalWindowExpressionCapabilities {
    pub row_number: Option<LeafCapability>,
    pub dense_rank: Option<LeafCapability>,
    pub ntile: Option<LeafCapability>,
    pub rank: Option<LeafCapability>,
    pub cume_dist: Option<LeafCapability>,
    pub percent_rank: Option<LeafCapability>,
}
// ANCHOR_END: RelationalWindowExpressionCapabilities
