use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeMap;

use crate::{
    Aggregate, Argument, ArgumentName, CollectionName, ComparisonOperatorName, FieldName,
    PathElement, RelationshipArgument, RelationshipName, VariableName,
};

// ANCHOR: Expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Expression")]
pub enum Expression {
    And {
        expressions: Vec<Expression>,
    },
    Or {
        expressions: Vec<Expression>,
    },
    Not {
        expression: Box<Expression>,
    },
    UnaryComparisonOperator {
        column: ComparisonTarget,
        operator: UnaryComparisonOperator,
    },
    BinaryComparisonOperator {
        column: ComparisonTarget,
        operator: ComparisonOperatorName,
        value: ComparisonValue,
    },
    ArrayComparison {
        column: ComparisonTarget,
        comparison: ArrayComparison,
    },
    Exists {
        in_collection: ExistsInCollection,
        predicate: Option<Box<Expression>>,
    },
}
// ANCHOR_END: Expression

// ANCHOR: ArrayComparison
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Array Comparison")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ArrayComparison {
    /// Check if the array contains the specified value
    Contains { value: ComparisonValue },
    /// Check is the array is empty
    IsEmpty,
}
// ANCHOR_END: ArrayComparison

// ANCHOR: UnaryComparisonOperator
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, JsonSchema,
)]
#[schemars(title = "Unary Comparison Operator")]
#[serde(rename_all = "snake_case")]
pub enum UnaryComparisonOperator {
    IsNull,
}
// ANCHOR_END: UnaryComparisonOperator

// ANCHOR: ComparisonTarget
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Comparison Target")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ComparisonTarget {
    Column {
        /// The name of the column
        name: FieldName,
        /// Arguments to satisfy the column specified by 'name'
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
        /// Path to a nested field within an object column
        field_path: Option<Vec<FieldName>>,
    },
    Aggregate {
        /// The aggregation method to use
        aggregate: Aggregate,
        /// Non-empty collection of relationships to traverse
        path: Vec<PathElement>,
    },
}
// ANCHOR_END: ComparisonTarget

// ANCHOR: ComparisonValue
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Comparison Value")]
pub enum ComparisonValue {
    Column {
        /// The name of the column
        name: FieldName,
        /// Arguments to satisfy the column specified by 'name'
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
        /// Path to a nested field within an object column
        field_path: Option<Vec<FieldName>>,
        /// Any relationships to traverse to reach this column
        #[serde(default)]
        path: Vec<PathElement>,
        /// The scope in which this column exists, identified
        /// by an top-down index into the stack of scopes.
        /// The stack grows inside each `Expression::Exists`,
        /// so scope 0 (the default) refers to the current collection,
        /// and each subsequent index refers to the collection outside
        /// its predecessor's immediately enclosing `Expression::Exists`
        /// expression.
        scope: Option<usize>,
    },
    Scalar {
        value: serde_json::Value,
    },
    Variable {
        name: VariableName,
    },
}
// ANCHOR_END: ComparisonValue

// ANCHOR: ExistsInCollection
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Exists In Collection")]
pub enum ExistsInCollection {
    Related {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        /// Path to a nested field within an object column that must be navigated
        /// before the relationship is navigated
        field_path: Option<Vec<FieldName>>,
        /// The name of the relationship to follow
        relationship: RelationshipName,
        /// Values to be provided to any collection arguments
        arguments: BTreeMap<ArgumentName, RelationshipArgument>,
    },
    Unrelated {
        /// The name of a collection
        collection: CollectionName,
        /// Values to be provided to any collection arguments
        arguments: BTreeMap<ArgumentName, RelationshipArgument>,
    },
    NestedCollection {
        column_name: FieldName,
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
        /// Path to a nested collection via object columns
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        field_path: Vec<FieldName>,
    },
    /// Specifies a column that contains a nested array of scalars. The
    /// array will be brought into scope of the nested expression where
    /// each element becomes an object with one '__value' column that
    /// contains the element value.
    NestedScalarCollection {
        column_name: FieldName,
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
        /// Path to a nested collection via object columns
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        field_path: Vec<FieldName>,
    },
}
// ANCHOR_END: ExistsInCollection
