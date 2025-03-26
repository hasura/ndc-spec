use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

mod capabilities;
pub use capabilities::*;
mod expression;
pub use expression::*;
mod types;
pub use types::*;

use crate::{CollectionName, FieldName, OrderDirection, ScopeName};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[schemars(title = "RelationalQuery")]
pub struct RelationalQuery {
    pub root_relation: Relation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "RelationalQueryResponse")]
pub struct RelationalQueryResponse(Vec<serde_json::Value>);

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[schemars(title = "Relation")]
pub enum Relation {
    From {
        collection: CollectionName,
        columns: Vec<FieldName>,
        scope_name: ScopeName,
    },
    Paginate {
        input: Box<Relation>,
        fetch: Option<u64>,
        skip: u64,
    },
    Project {
        input: Box<Relation>,
        exprs: IndexMap<FieldName, RelationalExpression>,
        scope_name: ScopeName,
    },
    Filter {
        input: Box<Relation>,
        predicate: RelationalExpression,
    },
    Sort {
        input: Box<Relation>,
        exprs: Vec<Sort>,
    },
    Join {
        left: Box<Relation>,
        left_scope_name: ScopeName,
        right: Box<Relation>,
        right_scope_name: ScopeName,
        on: Vec<JoinOn>,
        join_type: JoinType,
    },
    Aggregate {
        input: Box<Relation>,
        /// Only non-empty if the 'relational_query.aggregate.group_by' capability is supported.
        group_by: Vec<RelationalExpression>,
        aggregates: IndexMap<FieldName, RelationalExpression>,
        scope_name: ScopeName,
    },
    Window {
        input: Box<Relation>,
        exprs: IndexMap<FieldName, RelationalExpression>,
        scope_name: ScopeName,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Sort")]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "JoinOn")]
pub struct JoinOn {
    pub left: RelationalExpression,
    pub right: RelationalExpression,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash, Serialize, Deserialize, JsonSchema)]
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
