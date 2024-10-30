use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeMap;

use crate::{
    Aggregate, ArgumentName, CollectionName, Expression, Field, FieldName, Grouping, NestedField,
    OrderBy, ProcedureName, RelationshipName, VariableName,
};

// ANCHOR: QueryRequest
/// This is the request body of the query POST endpoint
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Query Request")]
pub struct QueryRequest {
    /// The name of a collection
    pub collection: CollectionName,
    /// The query syntax tree
    pub query: Query,
    /// Values to be provided to any collection arguments
    pub arguments: BTreeMap<ArgumentName, Argument>,
    /// Any relationships between collections involved in the query request
    pub collection_relationships: BTreeMap<RelationshipName, Relationship>,
    /// One set of named variables for each rowset to fetch. Each variable set
    /// should be subtituted in turn, and a fresh set of rows returned.
    pub variables: Option<Vec<BTreeMap<VariableName, serde_json::Value>>>,
}
// ANCHOR_END: QueryRequest

// ANCHOR: Query
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Query")]
pub struct Query {
    /// Aggregate fields of the query
    pub aggregates: Option<IndexMap<FieldName, Aggregate>>,
    /// Fields of the query
    pub fields: Option<IndexMap<FieldName, Field>>,
    /// Optionally limit to N results
    pub limit: Option<u32>,
    /// Optionally offset from the Nth result
    pub offset: Option<u32>,
    /// Optionally specify how rows should be ordered
    pub order_by: Option<OrderBy>,
    /// Optionally specify a predicate to apply to the rows
    pub predicate: Option<Expression>,
    /// Optionally group and aggregate the selected rows
    pub groups: Option<Grouping>,
}
// ANCHOR_END: Query

// ANCHOR: Argument
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Argument")]
pub enum Argument {
    /// The argument is provided by reference to a variable
    Variable { name: VariableName },
    /// The argument is provided as a literal value
    Literal { value: serde_json::Value },
}
// ANCHOR_END: Argument

// ANCHOR: Relationship
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relationship")]
pub struct Relationship {
    /// A mapping between columns on the source collection to columns on the target collection
    pub column_mapping: BTreeMap<FieldName, Vec<FieldName>>,
    pub relationship_type: RelationshipType,
    /// The name of a collection
    pub target_collection: CollectionName,
    /// Values to be provided to any collection arguments
    pub arguments: BTreeMap<ArgumentName, RelationshipArgument>,
}
// ANCHOR_END: Relationship

// ANCHOR: RelationshipArgument
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relationship Argument")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RelationshipArgument {
    /// The argument is provided by reference to a variable
    Variable {
        name: VariableName,
    },
    /// The argument is provided as a literal value
    Literal {
        value: serde_json::Value,
    },
    // The argument is provided based on a column of the source collection
    Column {
        name: FieldName,
    },
}
// ANCHOR_END: RelationshipArgument

// ANCHOR: RelationshipType
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, JsonSchema,
)]
#[schemars(title = "Relationship Type")]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    Object,
    Array,
}
// ANCHOR_END: RelationshipType

// ANCHOR: QueryResponse
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Query Response")]
/// Query responses may return multiple RowSets when using queries with variables.
/// Else, there should always be exactly one RowSet
pub struct QueryResponse(pub Vec<RowSet>);
// ANCHOR_END: QueryResponse

// ANCHOR: RowSet
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Row Set")]
pub struct RowSet {
    /// The results of the aggregates returned by the query
    pub aggregates: Option<IndexMap<FieldName, serde_json::Value>>,
    /// The rows returned by the query, corresponding to the query's fields
    pub rows: Option<Vec<IndexMap<FieldName, RowFieldValue>>>,
    /// The results of any grouping operation
    pub groups: Option<Vec<Group>>,
}
// ANCHOR_END: RowSet

// ANCHOR: Group
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Group")]
pub struct Group {
    /// Values of dimensions which identify this group
    pub dimensions: Vec<serde_json::Value>,
    /// Aggregates computed within this group
    pub aggregates: IndexMap<String, serde_json::Value>,
}
// ANCHOR_END: Group

// ANCHOR: RowFieldValue
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Row Field Value")]
pub struct RowFieldValue(pub serde_json::Value);

impl RowFieldValue {
    /// In the case where this field value was obtained using a
    /// [`Field::Relationship`], the returned JSON will be a [`RowSet`].
    /// We cannot express [`RowFieldValue`] as an enum, because
    /// [`RowFieldValue`] overlaps with values which have object types.
    pub fn as_rowset(self) -> Option<RowSet> {
        serde_json::from_value(self.0).ok()
    }
}
// ANCHOR_END: RowFieldValue

// ANCHOR: ExplainResponse
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Explain Response")]
pub struct ExplainResponse {
    /// A list of human-readable key-value pairs describing
    /// a query execution plan. For example, a connector for
    /// a relational database might return the generated SQL
    /// and/or the output of the `EXPLAIN` command. An API-based
    /// connector might encode a list of statically-known API
    /// calls which would be made.
    pub details: BTreeMap<String, String>,
}
// ANCHOR_END: ExplainResponse

// ANCHOR: MutationRequest
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Mutation Request")]
pub struct MutationRequest {
    /// The mutation operations to perform
    pub operations: Vec<MutationOperation>,
    /// The relationships between collections involved in the entire mutation request
    pub collection_relationships: BTreeMap<RelationshipName, Relationship>,
}
// ANCHOR_END: MutationRequest

// ANCHOR: MutationOperation
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Mutation Operation")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MutationOperation {
    Procedure {
        /// The name of a procedure
        name: ProcedureName,
        /// Any named procedure arguments
        arguments: BTreeMap<ArgumentName, serde_json::Value>,
        /// The fields to return from the result, or null to return everything
        fields: Option<NestedField>,
    },
}
// ANCHOR_END: MutationOperation

// ANCHOR: MutationResponse
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Mutation Response")]
pub struct MutationResponse {
    /// The results of each mutation operation, in the same order as they were received
    pub operation_results: Vec<MutationOperationResults>,
}
// ANCHOR_END: MutationResponse

// ANCHOR: MutationOperationResults
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Mutation Operation Results")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MutationOperationResults {
    Procedure { result: serde_json::Value },
}
// ANCHOR_END: MutationOperationResults

// ANCHOR: ErrorResponse
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Error Response")]
pub struct ErrorResponse {
    /// A human-readable summary of the error
    pub message: String,
    /// Any additional structured information about the error
    pub details: serde_json::Value,
}
// ANCHOR_END: ErrorResponse
