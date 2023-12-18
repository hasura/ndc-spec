use std::collections::BTreeMap;

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde_with::skip_serializing_none;

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

// ANCHOR_END
// ANCHOR: CapabilitiesResponse
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Capabilities Response")]
pub struct CapabilitiesResponse {
    pub versions: String,
    pub capabilities: Capabilities,
}
// ANCHOR_END: CapabilitiesResponse

// ANCHOR: LeafCapability
/// A unit value to indicate a particular leaf capability is supported.
/// This is an empty struct to allow for future sub-capabilities.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LeafCapability {}
// ANCHOR_END: LeafCapability

// ANCHOR: Capabilities
/// Describes the features of the specification which a data connector implements.
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Capabilities")]
pub struct Capabilities {
    pub query: QueryCapabilities,
    pub explain: Option<LeafCapability>,
    pub relationships: Option<RelationshipCapabilities>,
}
// ANCHOR_END: Capabilities

// ANCHOR: QueryCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Query Capabilities")]
pub struct QueryCapabilities {
    /// Does the connector support aggregate queries
    pub aggregates: Option<LeafCapability>,
    /// Does the connector support queries which use variables
    pub variables: Option<LeafCapability>,
}
// ANCHOR_END: QueryCapabilities

// ANCHOR: RelationshipCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relationship Capabilities")]
pub struct RelationshipCapabilities {
    /// Does the connector support comparisons that involve related collections (ie. joins)?
    pub relation_comparisons: Option<LeafCapability>,
    /// Does the connector support ordering by an aggregated array relationship?
    pub order_by_aggregate: Option<LeafCapability>,
}
// ANCHOR_END: RelationshipCapabilities

// ANCHOR: SchemaResponse
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Schema Response")]
pub struct SchemaResponse {
    /// A list of scalar types which will be used as the types of collection columns
    pub scalar_types: BTreeMap<String, ScalarType>,
    /// A list of object types which can be used as the types of arguments, or return types of procedures.
    /// Names should not overlap with scalar type names.
    pub object_types: BTreeMap<String, ObjectType>,
    /// Collections which are available for queries
    pub collections: Vec<CollectionInfo>,
    /// Functions (i.e. collections which return a single column and row)
    pub functions: Vec<FunctionInfo>,
    /// Procedures which are available for execution as part of mutations
    pub procedures: Vec<ProcedureInfo>,
}
// ANCHOR_END: SchemaResponse

// ANCHOR: ScalarType
/// The definition of a scalar type, i.e. types that can be used as the types of columns.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Scalar Type")]
pub struct ScalarType {
    /// A map from aggregate function names to their definitions. Result type names must be defined scalar types declared in ScalarTypesCapabilities.
    pub aggregate_functions: BTreeMap<String, AggregateFunctionDefinition>,
    /// A map from comparison operator names to their definitions. Argument type names must be defined scalar types declared in ScalarTypesCapabilities.
    pub comparison_operators: BTreeMap<String, ComparisonOperatorDefinition>,
}
// ANCHOR_END: ScalarType

// ANCHOR: ObjectType
/// The definition of an object type
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Object Type")]
pub struct ObjectType {
    /// Description of this type
    pub description: Option<String>,
    /// Fields defined on this object type
    pub fields: BTreeMap<String, ObjectField>,
}
// ANCHOR_END: ObjectType

// ANCHOR: ObjectField
/// The definition of an object field
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Object Field")]
pub struct ObjectField {
    /// Description of this field
    pub description: Option<String>,
    /// The type of this field
    #[serde(rename = "type")]
    pub r#type: Type,
}
// ANCHOR_END: ObjectField

// ANCHOR: Type
/// Types track the valid representations of values as JSON
#[derive(
    Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Type")]
pub enum Type {
    /// A named type
    Named {
        /// The name can refer to a primitive type or a scalar type
        name: String,
    },
    /// A nullable type
    Nullable {
        /// The type of the non-null inhabitants of this type
        underlying_type: Box<Type>,
    },
    /// An array type
    Array {
        /// The type of the elements of the array
        element_type: Box<Type>,
    },
}
// ANCHOR_END: Type

// ANCHOR: ComparisonOperatorDefinition
/// The definition of a comparison operator on a scalar type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Comparison Operator Definition")]
pub struct ComparisonOperatorDefinition {
    /// The type of the argument to this operator
    pub argument_type: Type,
}
// ANCHOR_END: ComparisonOperatorDefinition

// ANCHOR: AggregateFunctionDefinition
/// The definition of an aggregation function on a scalar type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Aggregate Function Definition")]
pub struct AggregateFunctionDefinition {
    /// The scalar or object type of the result of this function
    pub result_type: Type,
}
// ANCHOR_END: AggregateFunctionDefinition

// ANCHOR: CollectionInfo
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Collection Info")]
pub struct CollectionInfo {
    /// The name of the collection
    ///
    /// Note: these names are abstract - there is no requirement that this name correspond to
    /// the name of an actual collection in the database.
    pub name: String,
    /// Description of the collection
    pub description: Option<String>,
    /// Any arguments that this collection requires
    pub arguments: BTreeMap<String, ArgumentInfo>,
    /// The name of the collection's object type
    #[serde(rename = "type")]
    pub collection_type: String,
    /// Any uniqueness constraints enforced on this collection
    pub uniqueness_constraints: BTreeMap<String, UniquenessConstraint>,
    /// Any foreign key constraints enforced on this collection
    pub foreign_keys: BTreeMap<String, ForeignKeyConstraint>,
}
// ANCHOR_END: CollectionInfo

// ANCHOR: FunctionInfo
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Function Info")]
pub struct FunctionInfo {
    /// The name of the function
    pub name: String,
    /// Description of the function
    pub description: Option<String>,
    /// Any arguments that this collection requires
    pub arguments: BTreeMap<String, ArgumentInfo>,
    /// The name of the function's result type
    pub result_type: Type,
}
// ANCHOR_END: FunctionInfo

// ANCHOR: ArgumentInfo
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Argument Info")]
pub struct ArgumentInfo {
    /// Argument description
    pub description: Option<String>,
    /// The name of the type of this argument
    #[serde(rename = "type")]
    pub argument_type: Type,
}
// ANCHOR_END: ArgumentInfo

// ANCHOR: UniquenessConstraint
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Uniqueness Constraint")]
pub struct UniquenessConstraint {
    /// A list of columns which this constraint requires to be unique
    pub unique_columns: Vec<String>,
}
// ANCHOR_END: UniquenessConstraint

// ANCHOR: ForeignKeyConstraint
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Foreign Key Constraint")]
pub struct ForeignKeyConstraint {
    /// The columns on which you want want to define the foreign key.
    pub column_mapping: BTreeMap<String, String>,
    /// The name of a collection
    pub foreign_collection: String,
}
// ANCHOR_END: ForeignKeyConstraint

// ANCHOR: ProcedureInfo
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Procedure Info")]
pub struct ProcedureInfo {
    /// The name of the procedure
    pub name: String,
    /// Column description
    pub description: Option<String>,
    /// Any arguments that this collection requires
    pub arguments: BTreeMap<String, ArgumentInfo>,
    /// The name of the result type
    pub result_type: Type,
}
// ANCHOR_END: ProcedureInfo

// ANCHOR: QueryRequest
/// This is the request body of the query POST endpoint
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Query Request")]
pub struct QueryRequest {
    /// The name of a collection
    pub collection: String,
    /// The query syntax tree
    pub query: Query,
    /// Values to be provided to any collection arguments
    pub arguments: BTreeMap<String, Argument>,
    /// Any relationships between collections involved in the query request
    pub collection_relationships: BTreeMap<String, Relationship>,
    /// One set of named variables for each rowset to fetch. Each variable set
    /// should be subtituted in turn, and a fresh set of rows returned.
    pub variables: Option<Vec<BTreeMap<String, serde_json::Value>>>,
}
// ANCHOR_END: QueryRequest

// ANCHOR: Argument
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Argument")]
pub enum Argument {
    /// The argument is provided by reference to a variable
    Variable { name: String },
    /// The argument is provided as a literal value
    Literal { value: serde_json::Value },
}
// ANCHOR_END: Argument

// ANCHOR: Query
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Query")]
pub struct Query {
    /// Aggregate fields of the query
    pub aggregates: Option<IndexMap<String, Aggregate>>,
    /// Fields of the query
    pub fields: Option<IndexMap<String, Field>>,
    /// Optionally limit to N results
    pub limit: Option<u32>,
    /// Optionally offset from the Nth result
    pub offset: Option<u32>,
    pub order_by: Option<OrderBy>,
    #[serde(rename = "where")]
    pub predicate: Option<Expression>,
}
// ANCHOR_END: Query

// ANCHOR: Aggregate
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Aggregate")]
pub enum Aggregate {
    ColumnCount {
        /// The column to apply the count aggregate function to
        column: String,
        /// Whether or not only distinct items should be counted
        distinct: bool,
    },
    SingleColumn {
        /// The column to apply the aggregation function to
        column: String,
        /// Single column aggregate function name.
        function: String,
    },
    StarCount {},
}
// ANCHOR_END: Aggregate

// ANCHOR: NestedObject
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "NestedObject")]
pub struct NestedObject {
    pub fields: IndexMap<String, Field>,
}
// ANCHOR_END: NestedObject

// ANCHOR: NestedArray
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "NestedArray")]
pub struct NestedArray {
    pub fields: Box<Option<NestedField>>,
}
// ANCHOR_END: NestedArray

// ANCHOR: NestedField
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "NestedField")]
pub enum NestedField {
    Object(NestedObject),
    Array(NestedArray)
}
// ANCHOR_END: NestedField

// ANCHOR: Field
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Field")]
pub enum Field {
    Column {
        column: String,
        fields: Option<NestedField>
    },
    Relationship {
        query: Box<Query>,
        /// The name of the relationship to follow for the subquery
        relationship: String,
        /// Values to be provided to any collection arguments
        arguments: BTreeMap<String, RelationshipArgument>,
    },
}
// ANCHOR_END: Field

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
    Column {
        /// The name of the column
        name: String,
        /// Any relationships to traverse to reach this column
        path: Vec<PathElement>,
    },
    SingleColumnAggregate {
        /// The column to apply the aggregation function to
        column: String,
        /// Single column aggregate function name.
        function: String,
        /// Non-empty collection of relationships to traverse
        path: Vec<PathElement>,
    },
    StarCountAggregate {
        /// Non-empty collection of relationships to traverse
        path: Vec<PathElement>,
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
        operator: BinaryComparisonOperator,
        value: ComparisonValue,
    },
    BinaryArrayComparisonOperator {
        column: ComparisonTarget,
        operator: BinaryArrayComparisonOperator,
        values: Vec<ComparisonValue>,
    },
    Exists {
        in_collection: ExistsInCollection,
        #[serde(rename = "where")]
        predicate: Box<Expression>,
    },
}
// ANCHOR_END: Expression

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

// ANCHOR: BinaryArrayComparisonOperator
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, JsonSchema,
)]
#[schemars(title = "Binary Array Comparison Operator")]
#[serde(rename_all = "snake_case")]
pub enum BinaryArrayComparisonOperator {
    In,
}
// ANCHOR_END: BinaryArrayComparisonOperator

// ANCHOR: BinaryComparisonOperator
#[derive(
    Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Binary Comparison Operator")]
pub enum BinaryComparisonOperator {
    Equal,
    // should we rename this? To what?
    Other { name: String },
}
// ANCHOR_END: BinaryComparisonOperator

// ANCHOR: ComparisonTarget
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Comparison Target")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ComparisonTarget {
    Column {
        /// The name of the column
        name: String,
        /// Any relationships to traverse to reach this column
        path: Vec<PathElement>,
    },
    RootCollectionColumn {
        /// The name of the column
        name: String,
    },
}
// ANCHOR_END: ComparisonTarget

// ANCHOR: PathElement
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(title = "Path Element")]
pub struct PathElement {
    /// The name of the relationship to follow
    pub relationship: String,
    /// Values to be provided to any collection arguments
    pub arguments: BTreeMap<String, RelationshipArgument>,
    /// A predicate expression to apply to the target collection
    pub predicate: Box<Expression>,
}
// ANCHOR_END: PathElement

// ANCHOR: ComparisonValue
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Comparison Value")]
pub enum ComparisonValue {
    Column { column: ComparisonTarget },
    Scalar { value: serde_json::Value },
    Variable { name: String },
}
// ANCHOR_END: ComparisonValue

// ANCHOR: ExistsInCollection
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Exists In Collection")]
pub enum ExistsInCollection {
    Related {
        relationship: String,
        /// Values to be provided to any collection arguments
        arguments: BTreeMap<String, RelationshipArgument>,
    },
    Unrelated {
        /// The name of a collection
        collection: String,
        /// Values to be provided to any collection arguments
        arguments: BTreeMap<String, RelationshipArgument>,
    },
}
// ANCHOR_END: ExistsInCollection

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
    pub aggregates: Option<IndexMap<String, serde_json::Value>>,
    /// The rows returned by the query, corresponding to the query's fields
    pub rows: Option<Vec<IndexMap<String, RowFieldValue>>>,
}
// ANCHOR_END: RowSet

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
    pub collection_relationships: BTreeMap<String, Relationship>,
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
        name: String,
        /// Any named procedure arguments
        arguments: BTreeMap<String, serde_json::Value>,
        /// The fields to return
        fields: Option<IndexMap<String, Field>>,
    },
}
// ANCHOR_END: MutationOperation

// ANCHOR: Relationship
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relationship")]
pub struct Relationship {
    /// A mapping between columns on the source collection to columns on the target collection
    pub column_mapping: BTreeMap<String, String>,
    pub relationship_type: RelationshipType,
    /// The name of a collection
    pub target_collection: String,
    /// Values to be provided to any collection arguments
    pub arguments: BTreeMap<String, RelationshipArgument>,
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
        name: String,
    },
    /// The argument is provided as a literal value
    Literal {
        value: serde_json::Value,
    },
    // The argument is provided based on a column of the source collection
    Column {
        name: String,
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

// ANCHOR: MutationResponse
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Mutation Response")]
pub struct MutationResponse {
    /// The results of each mutation operation, in the same order as they were received
    pub operation_results: Vec<MutationOperationResults>,
}
// ANCHOR_END: MutationResponse

// ANCHOR: MutationOperationResults
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Mutation Operation Results")]
pub struct MutationOperationResults {
    /// The number of rows affected by the mutation operation
    pub affected_rows: u32,
    /// The rows affected by the mutation operation
    pub returning: Option<Vec<IndexMap<String, RowFieldValue>>>,
}
// ANCHOR_END: MutationOperationResults

#[cfg(test)]
mod tests {
    use crate::models;
    use goldenfile::Mint;
    use schemars::schema_for;
    use std::{io::Write, path::PathBuf};

    #[test]
    fn test_json_schemas() {
        let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

        let mut mint = Mint::new(test_dir);

        test_json_schema(
            &mut mint,
            schema_for!(models::ErrorResponse),
            "error_response.jsonschema",
        );

        test_json_schema(
            &mut mint,
            schema_for!(models::SchemaResponse),
            "schema_response.jsonschema",
        );

        test_json_schema(
            &mut mint,
            schema_for!(models::CapabilitiesResponse),
            "capabilities_response.jsonschema",
        );

        test_json_schema(
            &mut mint,
            schema_for!(models::QueryRequest),
            "query_request.jsonschema",
        );
        test_json_schema(
            &mut mint,
            schema_for!(models::QueryResponse),
            "query_response.jsonschema",
        );

        test_json_schema(
            &mut mint,
            schema_for!(models::ExplainResponse),
            "explain_response.jsonschema",
        );

        test_json_schema(
            &mut mint,
            schema_for!(models::MutationRequest),
            "mutation_request.jsonschema",
        );
        test_json_schema(
            &mut mint,
            schema_for!(models::MutationResponse),
            "mutation_response.jsonschema",
        );
    }

    fn test_json_schema(mint: &mut Mint, schema: schemars::schema::RootSchema, filename: &str) {
        let expected_path = PathBuf::from_iter(["json_schema", filename]);

        let mut expected = mint.new_goldenfile(expected_path).unwrap();

        write!(
            expected,
            "{}",
            serde_json::to_string_pretty(&schema).unwrap()
        )
        .unwrap();
    }
}
