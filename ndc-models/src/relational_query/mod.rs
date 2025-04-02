use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

mod capabilities;
pub use capabilities::*;
mod expression;
pub use expression::*;
mod types;
pub use types::*;

use crate::{CollectionName, FieldName, OrderDirection};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(title = "RelationalQuery")]
pub struct RelationalQuery {
    pub root_relation: Relation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "RelationalQueryResponse")]
#[serde(rename_all = "snake_case")]
pub struct RelationalQueryResponse {
    pub rows: Vec<Vec<serde_json::Value>>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Relation")]
pub enum Relation {
    From {
        collection: CollectionName,
        columns: Vec<FieldName>,
    },
    Paginate {
        #[cfg(not(feature = "arc-relation"))]
        input: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        input: std::sync::Arc<Relation>,

        fetch: Option<u64>,
        skip: u64,
    },
    Project {
        #[cfg(not(feature = "arc-relation"))]
        input: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        input: std::sync::Arc<Relation>,

        exprs: Vec<RelationalExpression>,
    },
    Filter {
        #[cfg(not(feature = "arc-relation"))]
        input: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        input: std::sync::Arc<Relation>,

        predicate: RelationalExpression,
    },
    Sort {
        #[cfg(not(feature = "arc-relation"))]
        input: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        input: std::sync::Arc<Relation>,

        exprs: Vec<Sort>,
    },
    Join {
        #[cfg(not(feature = "arc-relation"))]
        left: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        left: std::sync::Arc<Relation>,

        #[cfg(not(feature = "arc-relation"))]
        right: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        right: std::sync::Arc<Relation>,

        on: Vec<JoinOn>,
        join_type: JoinType,
    },
    Aggregate {
        #[cfg(not(feature = "arc-relation"))]
        input: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        input: std::sync::Arc<Relation>,

        /// Only non-empty if the 'relational_query.aggregate.group_by' capability is supported.
        group_by: Vec<RelationalExpression>,
        aggregates: Vec<RelationalExpression>,
    },
    Window {
        #[cfg(not(feature = "arc-relation"))]
        input: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        input: std::sync::Arc<Relation>,

        exprs: Vec<RelationalExpression>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Sort", rename_all = "snake_case")]
pub struct Sort {
    pub expr: RelationalExpression,
    pub direction: OrderDirection,
    pub nulls_sort: NullsSort,
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, JsonSchema,
)]
#[schemars(title = "Nulls Sort")]
#[serde(rename_all = "snake_case")]
pub enum NullsSort {
    NullsFirst,
    NullsLast,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "JoinOn", rename_all = "snake_case")]
pub struct JoinOn {
    pub left: RelationalExpression,
    pub right: RelationalExpression,
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[schemars(title = "JoinType")]
pub enum JoinType {
    /// Only used when the capability `relational_query.join.join_types.left` is supported.
    Left,
    /// Only used when the capability `relational_query.join.join_types.right` is supported.
    Right,
    /// Only used when the capability `relational_query.join.join_types.inner` is supported.
    Inner,
    /// Only used when the capability `relational_query.join.join_types.full` is supported.
    Full,
}
