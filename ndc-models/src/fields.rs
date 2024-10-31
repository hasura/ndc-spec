use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{
    Argument, ArgumentName, Expression, FieldName, Query, RelationshipArgument, RelationshipName,
};

// ANCHOR: Field
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Field")]
pub enum Field {
    Column {
        column: FieldName,
        /// When the type of the column is a (possibly-nullable) array or object,
        /// the caller can request a subset of the complete column data,
        /// by specifying fields to fetch here.
        /// If omitted, the column data will be fetched in full.
        fields: Option<NestedField>,
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
    },
    Relationship {
        query: Box<Query>,
        /// The name of the relationship to follow for the subquery
        relationship: RelationshipName,
        /// Values to be provided to any collection arguments
        arguments: BTreeMap<ArgumentName, RelationshipArgument>,
    },
}
// ANCHOR_END: Field

// ANCHOR: NestedObject
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(title = "NestedObject")]
pub struct NestedObject {
    pub fields: IndexMap<FieldName, Field>,
}
// ANCHOR_END: NestedObject

// ANCHOR: NestedArray
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(title = "NestedArray")]
pub struct NestedArray {
    pub fields: Box<NestedField>,
}
// ANCHOR_END: NestedArray

// ANCHOR: NestedCollection
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(title = "NestedCollection")]
pub struct NestedCollection {
    pub query: Query,
}
// ANCHOR_END: NestedCollection

// ANCHOR: NestedField
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "NestedField")]
pub enum NestedField {
    Object(NestedObject),
    Array(NestedArray),
    Collection(NestedCollection),
}
// ANCHOR_END: NestedField

// ANCHOR: PathElement
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(title = "Path Element")]
pub struct PathElement {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    /// Path to a nested field within an object column that must be navigated
    /// before the relationship is navigated
    pub field_path: Option<Vec<FieldName>>,
    /// The name of the relationship to follow
    pub relationship: RelationshipName,
    /// Values to be provided to any collection arguments
    pub arguments: BTreeMap<ArgumentName, RelationshipArgument>,
    /// A predicate expression to apply to the target collection
    pub predicate: Option<Box<Expression>>,
}
// ANCHOR_END: PathElement
