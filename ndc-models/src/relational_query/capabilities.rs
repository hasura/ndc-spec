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
    pub union: Option<LeafCapability>,
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
    pub left_semi: Option<LeafCapability>,
    pub left_anti: Option<LeafCapability>,
    pub right_semi: Option<LeafCapability>,
    pub right_anti: Option<LeafCapability>,
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
    pub conditional: RelationalConditionalExpressionCapabilities,
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
    pub nullif: Option<LeafCapability>,
}
// ANCHOR_END: RelationalConditionalExpressionCapabilities

// ANCHOR: RelationalFilterExpressionCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Filter Expression Capabilities")]
pub struct RelationalComparisonExpressionCapabilities {
    pub between: Option<LeafCapability>,
    pub contains: Option<LeafCapability>,
    pub greater_than_eq: Option<LeafCapability>,
    pub greater_than: Option<LeafCapability>,
    pub ilike: Option<LeafCapability>,
    pub in_list: Option<LeafCapability>,
    pub is_false: Option<LeafCapability>,
    pub is_nan: Option<LeafCapability>,
    pub is_null: Option<LeafCapability>,
    pub is_true: Option<LeafCapability>,
    pub is_zero: Option<LeafCapability>,
    pub less_than_eq: Option<LeafCapability>,
    pub less_than: Option<LeafCapability>,
    pub like: Option<LeafCapability>,
}
// ANCHOR_END: RelationalFilterExpressionCapabilities

// ANCHOR: RelationalScalarExpressionCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Scalar Expression Capabilities")]
pub struct RelationalScalarExpressionCapabilities {
    pub abs: Option<LeafCapability>,
    pub and: Option<LeafCapability>,
    pub array_element: Option<LeafCapability>,
    pub binary_concat: Option<LeafCapability>,
    pub btrim: Option<LeafCapability>,
    pub ceil: Option<LeafCapability>,
    pub character_length: Option<LeafCapability>,
    pub coalesce: Option<LeafCapability>,
    pub concat: Option<LeafCapability>,
    pub cos: Option<LeafCapability>,
    pub current_date: Option<LeafCapability>,
    pub current_time: Option<LeafCapability>,
    pub current_timestamp: Option<LeafCapability>,
    pub date_part: Option<DatePartScalarExpressionCapability>,
    pub date_trunc: Option<LeafCapability>,
    pub divide: Option<LeafCapability>,
    pub exp: Option<LeafCapability>,
    pub floor: Option<LeafCapability>,
    pub get_field: Option<LeafCapability>,
    pub greatest: Option<LeafCapability>,
    pub least: Option<LeafCapability>,
    pub left: Option<LeafCapability>,
    pub ln: Option<LeafCapability>,
    pub log: Option<LeafCapability>,
    pub log10: Option<LeafCapability>,
    pub log2: Option<LeafCapability>,
    pub lpad: Option<LeafCapability>,
    pub ltrim: Option<LeafCapability>,
    pub minus: Option<LeafCapability>,
    pub modulo: Option<LeafCapability>,
    pub multiply: Option<LeafCapability>,
    pub negate: Option<LeafCapability>,
    pub not: Option<LeafCapability>,
    pub nvl: Option<LeafCapability>,
    pub or: Option<LeafCapability>,
    pub plus: Option<LeafCapability>,
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
    pub substr_index: Option<LeafCapability>,
    pub substr: Option<LeafCapability>,
    pub tan: Option<LeafCapability>,
    pub to_date: Option<LeafCapability>,
    pub to_lower: Option<LeafCapability>,
    pub to_timestamp: Option<LeafCapability>,
    pub to_upper: Option<LeafCapability>,
    pub trunc: Option<LeafCapability>,
}
// ANCHOR_END: RelationalScalarExpressionCapabilities

// ANCHOR: DatePartScalarExpressionCapability
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Date Part Scalar Expression Capability")]
pub struct DatePartScalarExpressionCapability {
    pub year: Option<LeafCapability>,
    pub quarter: Option<LeafCapability>,
    pub month: Option<LeafCapability>,
    pub week: Option<LeafCapability>,
    pub day_of_week: Option<LeafCapability>,
    pub day_of_year: Option<LeafCapability>,
    pub day: Option<LeafCapability>,
    pub hour: Option<LeafCapability>,
    pub minute: Option<LeafCapability>,
    pub second: Option<LeafCapability>,
    pub microsecond: Option<LeafCapability>,
    pub millisecond: Option<LeafCapability>,
    pub nanosecond: Option<LeafCapability>,
    pub epoch: Option<LeafCapability>,
}
// ANCHOR_END: DatePartScalarExpressionCapability

// ANCHOR: RelationalAggregateExpressionCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Aggregate Expression Capabilities")]
pub struct RelationalAggregateExpressionCapabilities {
    pub avg: Option<LeafCapability>,
    pub bool_and: Option<LeafCapability>,
    pub bool_or: Option<LeafCapability>,
    pub count: Option<RelationalAggregateFunctionCapabilities>,
    pub first_value: Option<LeafCapability>,
    pub last_value: Option<LeafCapability>,
    pub max: Option<LeafCapability>,
    pub median: Option<LeafCapability>,
    pub min: Option<LeafCapability>,
    pub string_agg: Option<LeafCapability>,
    pub sum: Option<LeafCapability>,
    pub var: Option<LeafCapability>,
    pub stddev: Option<LeafCapability>,
    pub stddev_pop: Option<LeafCapability>,
    pub approx_percentile_cont: Option<LeafCapability>,
    pub array_agg: Option<LeafCapability>,
}
// ANCHOR_END: RelationalAggregateExpressionCapabilities

// ANCHOR: RelationalAggregateFunctionCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relational Aggregate Function Capabilities")]
pub struct RelationalAggregateFunctionCapabilities {
    pub distinct: Option<LeafCapability>,
}
// ANCHOR_END: RelationalAggregateFunctionCapabilities

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
