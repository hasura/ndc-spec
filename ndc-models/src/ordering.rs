use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeMap;

use crate::{Aggregate, Argument, ArgumentName, FieldName, PathElement};

// ANCHOR: OrderBy
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Order By")]
pub struct OrderBy {
    /// The elements to order by, in priority order
    pub elements: Vec<OrderByElement>,
}
// ANCHOR_END: OrderBy

// ANCHOR: OrderByElement
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Order By Element")]
pub struct OrderByElement {
    pub order_direction: OrderDirection,
    pub target: OrderByTarget,
}
// ANCHOR_END: OrderByElement

// ANCHOR: OrderByTarget
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Order By Target")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrderByTarget {
    /// The ordering is performed over a column.
    Column {
        /// Any (object) relationships to traverse to reach this column.
        /// Only non-empty if the 'relationships' capability is supported.
        path: Vec<PathElement>,
        /// The name of the column
        name: FieldName,
        /// Arguments to satisfy the column specified by 'name'
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
        /// Path to a nested field within an object column.
        /// Only non-empty if the 'query.nested_fields.order_by' capability is supported.
        field_path: Option<Vec<FieldName>>,
    },
    /// The ordering is performed over the result of an aggregation.
    /// Only used if the 'relationships.order_by_aggregate' capability is supported.
    Aggregate {
        /// Non-empty collection of relationships to traverse
        path: Vec<PathElement>,
        /// The aggregation method to use
        aggregate: Aggregate,
    },
}
// ANCHOR_END: OrderByTarget

// ANCHOR: OrderDirection
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, JsonSchema,
)]
#[schemars(title = "Order Direction")]
#[serde(rename_all = "snake_case")]
pub enum OrderDirection {
    Asc,
    Desc,
}
// ANCHOR_END: OrderDirection
