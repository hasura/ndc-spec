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
    /// A comparison against a nested array column.
    /// Only used if the 'query.nested_fields.filter_by.nested_arrays' capability is supported.
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
    /// Check if the array contains the specified value.
    /// Only used if the 'query.nested_fields.filter_by.nested_arrays.contains' capability is supported.
    Contains { value: ComparisonValue },
    /// Check is the array is empty.
    /// Only used if the 'query.nested_fields.filter_by.nested_arrays.is_empty' capability is supported.
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
    /// The comparison targets a column.
    Column {
        /// The name of the column
        name: FieldName,
        /// Arguments to satisfy the column specified by 'name'
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
        /// Path to a nested field within an object column.
        /// Only non-empty if the 'query.nested_fields.filter_by' capability is supported.
        field_path: Option<Vec<FieldName>>,
    },
    /// The comparison targets the result of aggregation.
    /// Only used if the 'query.aggregates.filter_by' capability is supported.
    Aggregate {
        /// Non-empty collection of relationships to traverse
        path: Vec<PathElement>,
        /// The aggregation method to use
        aggregate: Aggregate,
    },
}
// ANCHOR_END: ComparisonTarget

// ANCHOR: ComparisonValue
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Comparison Value")]
pub enum ComparisonValue {
    /// The value to compare against should be drawn from another column
    Column {
        /// Any relationships to traverse to reach this column.
        /// Only non-empty if the 'relationships.relation_comparisons' is supported.
        path: Vec<PathElement>,
        /// The name of the column
        name: FieldName,
        /// Arguments to satisfy the column specified by 'name'
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
        /// Path to a nested field within an object column.
        /// Only non-empty if the 'query.nested_fields.filter_by' capability is supported.
        field_path: Option<Vec<FieldName>>,
        /// The scope in which this column exists, identified
        /// by an top-down index into the stack of scopes.
        /// The stack grows inside each `Expression::Exists`,
        /// so scope 0 (the default) refers to the current collection,
        /// and each subsequent index refers to the collection outside
        /// its predecessor's immediately enclosing `Expression::Exists`
        /// expression.
        /// Only used if the 'query.exists.named_scopes' capability is supported.
        scope: Option<usize>,
    },
    /// A scalar value to compare against
    Scalar { value: serde_json::Value },
    /// A value to compare against that is to be drawn from the query's variables.
    /// Only used if the 'query.variables' capability is supported.
    Variable { name: VariableName },
}
// ANCHOR_END: ComparisonValue

// ANCHOR: ExistsInCollection
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Exists In Collection")]
pub enum ExistsInCollection {
    /// The rows to evaluate the exists predicate against come from a related collection.
    /// Only used if the 'relationships' capability is supported.
    Related {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        /// Path to a nested field within an object column that must be navigated
        /// before the relationship is navigated
        /// Only non-empty if the 'relationships.nested' capability is supported.
        field_path: Option<Vec<FieldName>>,
        /// The name of the relationship to follow
        relationship: RelationshipName,
        /// Values to be provided to any collection arguments
        arguments: BTreeMap<ArgumentName, RelationshipArgument>,
    },
    /// The rows to evaluate the exists predicate against come from an unrelated collection
    /// Only used if the 'query.exists.unrelated' capability is supported.
    Unrelated {
        /// The name of a collection
        collection: CollectionName,
        /// Values to be provided to any collection arguments
        arguments: BTreeMap<ArgumentName, RelationshipArgument>,
    },
    /// The rows to evaluate the exists predicate against come from a nested array field.
    /// Only used if the 'query.exists.nested_collections' capability is supported.
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
    /// Only used if the 'query.exists.nested_scalar_collections' capability is supported.
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
