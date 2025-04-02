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
    /// Translates to SQL approximately:
    /// ```sql
    /// SELECT <scope_name>.<column0> [, <scope_name>.<columnN> ...]
    /// FROM <collection> AS <scope_name>
    /// ```
    /// Introduces a new scope named `scope_name` that `columns` will be available on.
    From {
        collection: CollectionName,
        columns: Vec<FieldName>,
        scope_name: ScopeName,
    },
    /// Translates to SQL approximately:
    /// ```sql
    /// <input>
    /// [LIMIT <fetch>] OFFSET <skip>
    /// ```
    /// Does not introduce a new scope.
    Paginate {
        #[cfg(not(feature = "arc-relation"))]
        input: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        input: std::sync::Arc<Relation>,

        fetch: Option<u64>,
        skip: u64,
    },
    /// Translates to SQL approximately:
    /// ```sql
    /// SELECT *
    /// FROM (
    ///     SELECT <expr0> AS <expr0_field_name> [, <exprN> AS <exprN_field_name> ...]
    ///     <input>
    /// ) AS <scope_name>
    ///
    /// ```
    /// `exprs` will refer to scopes defined inside input.
    /// This introduces a new scope named `scope_name` that hides the scopes defined inside input.
    /// The projected `exprs` will be available on this new scope for relations that wrap this one.
    Project {
        #[cfg(not(feature = "arc-relation"))]
        input: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        input: std::sync::Arc<Relation>,

        exprs: Vec<NamedRelationalExpression>,
        scope_name: ScopeName,
    },
    /// Translates to SQL approximately:
    /// ```sql
    /// <input>
    /// WHERE <predicate>
    /// ```
    /// This does not introduce a new scope, so `predicate` exprs will refer to the scopes defined inside input
    Filter {
        #[cfg(not(feature = "arc-relation"))]
        input: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        input: std::sync::Arc<Relation>,

        predicate: RelationalExpression,
    },
    /// Translates to SQL approximately:
    /// ```sql
    /// <input>
    /// ORDER BY <exprs0.expr> <exprs0.direction> <exprs0.nulls_sort: NULLS FIRST|LAST> [, <exprsN.expr> <exprsN.direction> <exprsN.nulls_sort: NULLS FIRST|LAST> ...]
    /// ```
    /// This does not introduce a new scope, so `sort` exprs will refer to the scopes defined inside input
    Sort {
        #[cfg(not(feature = "arc-relation"))]
        input: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        input: std::sync::Arc<Relation>,

        exprs: Vec<Sort>,
    },
    /// Translates to SQL approximately:
    /// ```sql
    /// <left>
    /// <join_type> JOIN <right> AS <right_scope_name>
    /// ON <on0.left> = <on0.right> [, <onN.left> = <onN.right> ...>
    /// ```
    /// Introduces a new scope for the right relation (right_scope_name) without hiding the left relation's scope.
    /// `on` exprs will refer to both the scope defined inside left and the right_scope_name.
    Join {
        #[cfg(not(feature = "arc-relation"))]
        left: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        left: std::sync::Arc<Relation>,

        #[cfg(not(feature = "arc-relation"))]
        right: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        right: std::sync::Arc<Relation>,

        right_scope_name: ScopeName,

        on: Vec<JoinOn>,
        join_type: JoinType,
    },
    /// Translates to SQL approximately:
    /// ```sql
    /// SELECT *
    /// FROM (
    ///     SELECT <aggregates0.expr> AS <aggregates0.field_name> [, <aggregatesN.expr> AS <aggregatesN.field_name> ...]
    ///     <input>
    ///     [GROUP BY <group_by0> [, <group_byN> ...]]
    /// ) AS <scope_name>
    /// ```
    /// `aggregates` and `group_by` expressions will refer to scopes defined inside input.
    /// This introduces a new scope named `scope_name` that hides the scopes defined inside input.
    /// The calculated `aggregates` will be available on this new scope for relations that wrap this one.
    Aggregate {
        #[cfg(not(feature = "arc-relation"))]
        input: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        input: std::sync::Arc<Relation>,

        /// Only non-empty if the 'relational_query.aggregate.group_by' capability is supported.
        group_by: Vec<NamedRelationalExpression>,
        aggregates: Vec<NamedRelationalExpression>,
        scope_name: ScopeName,
    },
    /// Translates to SQL approximately:
    /// ```sql
    /// SELECT *
    /// FROM (
    ///     SELECT <exprs0> AS <exprs0_field_name> [, <exprsN> AS <exprsN_field_name> ...]
    ///     <input>
    /// ) AS <scope_name>
    /// ```
    /// `exprs` will refer to scopes defined inside input.
    /// This introduces a new scope named `scope_name` that hides the scopes defined inside input.
    /// The projected `exprs` will be available on this new scope for relations that wrap this one.
    //
    // TODO: This is busted because in DF it doesn't create a new scope and just adds to the existing one.
    // But we really do need to create a new scope because we don't have "an existing scope" when multiple scopes are in play
    // How about combining this with Project with another property? Then we could project and window in one step,
    // which lets us create a new scope.
    Window {
        #[cfg(not(feature = "arc-relation"))]
        input: Box<Relation>,
        #[cfg(feature = "arc-relation")]
        input: std::sync::Arc<Relation>,

        exprs: Vec<NamedRelationalExpression>,
        scope_name: ScopeName,
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
