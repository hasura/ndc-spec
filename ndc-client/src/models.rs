use std::collections::BTreeMap;

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde_with::skip_serializing_none;

// ANCHOR_END
// ANCHOR: CapabilitiesResponse
// ANCHOR: CapabilitiesResponse
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CapabilitiesResponse {
    pub versions: String,
    pub capabilities: Capabilities,
}
// ANCHOR_END: CapabilitiesResponse

// ANCHOR: Capabilities
/// Describes the features of the specification which a data connector implements.
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Capabilities {
    pub query: Option<QueryCapabilities>,
    pub explain: Option<serde_json::Value>,
    pub mutations: Option<MutationCapabilities>,
    pub relationships: Option<serde_json::Value>,
}
// ANCHOR_END: Capabilities

// ANCHOR: QueryCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct QueryCapabilities {
    /// Does the agent support comparisons that involve related collections (ie. joins)?
    pub relation_comparisons: Option<serde_json::Value>,
    /// Does the agent support ordering by an aggregated array relationship?
    pub order_by_aggregate: Option<serde_json::Value>,
    /// Does the agent support foreach queries, i.e. queries with variables
    pub foreach: Option<serde_json::Value>,
}
// ANCHOR_END: QueryCapabilities

// ANCHOR: MutationCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MutationCapabilities {
    /// Whether or not nested inserts to related collections are supported
    pub nested_inserts: Option<serde_json::Value>,
    pub returning: Option<serde_json::Value>,
}
// ANCHOR_END: MutationCapabilities

// ANCHOR: SchemaResponse
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SchemaResponse {
    /// A list of scalar types which will be used as the types of collection columns
    pub scalar_types: BTreeMap<String, ScalarType>,
    /// A list of object types which can be used as the types of arguments, or return types of procedures.
    /// Names should not overlap with collection names or scalar type names.
    pub object_types: BTreeMap<String, ObjectType>,
    /// Collections which are available for queries and/or mutations
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
pub struct ScalarType {
    /// A map from aggregate function names to their definitions. Result type names must be defined scalar types declared in ScalarTypesCapabilities.
    pub aggregate_functions: BTreeMap<String, AggregateFunctionDefinition>,
    /// A map from comparison operator names to their definitions. Argument type names must be defined scalar types declared in ScalarTypesCapabilities.
    pub comparison_operators: BTreeMap<String, ComparisonOperatorDefinition>,
    /// A map from update operator names to their definitions.
    pub update_operators: BTreeMap<String, UpdateOperatorDefinition>,
}
// ANCHOR_END: ScalarType

// ANCHOR: ObjectType
/// The definition of an object type
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
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
pub struct ObjectField {
    /// Description of this field
    pub description: Option<String>,
    /// Any arguments that this object field accepts
    pub arguments: BTreeMap<String, ArgumentInfo>,
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
pub struct ComparisonOperatorDefinition {
    /// The type of the argument to this operator
    pub argument_type: Type,
}
// ANCHOR_END: ComparisonOperatorDefinition

// ANCHOR: AggregateFunctionDefinition
/// The definition of an aggregation function on a scalar type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AggregateFunctionDefinition {
    /// The scalar or object type of the result of this function
    pub result_type: Type,
}
// ANCHOR_END: AggregateFunctionDefinition

// ANCHOR: UpdateOperatorDefinition
/// The definition of an update operator on a scalar type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct UpdateOperatorDefinition {
    /// The type of the argument to this operator
    pub argument_type: Type,
}
// ANCHOR_END: UpdateOperatorDefinition

// ANCHOR: CollectionInfo
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
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
    /// The set of names of insercollection columns, or null if inserts are not supported
    pub insertable_columns: Option<Vec<String>>,
    /// The set of names of updateable columns, or null if updates are not supported
    pub updatable_columns: Option<Vec<String>>,
    /// Whether or not existing rows can be deleted from the collection
    pub deletable: bool,
    /// Any uniqueness constraints enforced on this collection
    pub uniqueness_constraints: BTreeMap<String, UniquenessConstraint>,
    /// Any foreign key constraints enforced on this collection
    pub foreign_keys: BTreeMap<String, ForeignKeyConstraint>,
}
// ANCHOR_END: CollectionInfo

// ANCHOR: FunctionInfo
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
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
pub struct UniquenessConstraint {
    /// A list of columns which this constraint requires to be unique
    pub unique_columns: Vec<String>,
}
// ANCHOR_END: UniquenessConstraint

// ANCHOR: ForeignKeyConstraint
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
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
pub enum Aggregate {
    // TODO: do we need aggregation row limits?
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

// ANCHOR: Field
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Field {
    Column {
        column: String,
        /// Values to be provided to any field arguments
        arguments: BTreeMap<String, Argument>,
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
pub struct OrderBy {
    /// The elements to order by, in priority order
    pub elements: Vec<OrderByElement>,
}
// ANCHOR_END: OrderBy

// ANCHOR: OrderByElement
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OrderByElement {
    pub order_direction: OrderDirection,
    pub target: OrderByTarget,
}
// ANCHOR_END: OrderByElement

// ANCHOR: OrderByTarget
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
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
#[serde(rename_all = "snake_case")]
pub enum OrderDirection {
    Asc,
    Desc,
}
// ANCHOR_END: OrderDirection

// ANCHOR: Expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
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
        column: Box<ComparisonTarget>,
        operator: Box<UnaryComparisonOperator>,
    },
    BinaryComparisonOperator {
        column: Box<ComparisonTarget>,
        operator: Box<BinaryComparisonOperator>,
        value: Box<ComparisonValue>,
    },
    BinaryArrayComparisonOperator {
        column: Box<ComparisonTarget>,
        operator: Box<BinaryArrayComparisonOperator>,
        values: Vec<ComparisonValue>,
    },
    Exists {
        in_collection: Box<ExistsInCollection>,
        #[serde(rename = "where")]
        predicate: Box<Expression>,
    },
}
// ANCHOR_END: Expression

// ANCHOR: UnaryComparisonOperator
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum UnaryComparisonOperator {
    IsNull,
}
// ANCHOR_END: UnaryComparisonOperator

// ANCHOR: BinaryArrayComparisonOperator
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, JsonSchema,
)]
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
pub enum BinaryComparisonOperator {
    Equal,
    // should we rename this? To what?
    Other { name: String },
}
// ANCHOR_END: BinaryComparisonOperator

// ANCHOR: ComparisonTarget
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
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
pub enum ComparisonValue {
    Column { column: Box<ComparisonTarget> },
    Scalar { value: serde_json::Value },
    Variable { name: String },
}
// ANCHOR_END: ComparisonValue

// ANCHOR: ExistsInCollection
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
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
/// Query responses may return multiple RowSets when using foreach queries
/// Else, there should always be exactly one RowSet
pub struct QueryResponse(pub Vec<RowSet>);
// ANCHOR_END: QueryResponse

// ANCHOR: RowSet
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RowSet {
    /// The results of the aggregates returned by the query
    pub aggregates: Option<IndexMap<String, serde_json::Value>>,
    /// The rows returned by the query, corresponding to the query's fields
    pub rows: Option<Vec<IndexMap<String, RowFieldValue>>>,
}
// ANCHOR_END: RowSet

// ANCHOR: RowFieldValue
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum RowFieldValue {
    Relationship(RowSet),
    Column(serde_json::Value),
}
// ANCHOR_END: RowFieldValue

// ANCHOR: ExplainResponse
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExplainResponse {
    /// Lines of the formatted explain plan response
    pub lines: Vec<String>,
    /// The generated query - i.e. SQL for a relational DB
    pub query: String,
}
// ANCHOR_END: ExplainResponse

// ANCHOR: MutationRequest
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MutationRequest {
    /// The schema by which to interpret row data specified in any insert operations in this request
    pub insert_schema: Vec<CollectionInsertSchema>,
    /// The mutation operations to perform
    pub operations: Vec<MutationOperation>,
    /// The relationships between collections involved in the entire mutation request
    pub collection_relationships: BTreeMap<String, Relationship>,
}
// ANCHOR_END: MutationRequest

// ANCHOR: CollectionInsertSchema
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CollectionInsertSchema {
    /// The fields that will be found in the insert row data for the collection and the schema for each field
    pub fields: BTreeMap<String, InsertFieldSchema>,
    /// The name of a collection
    pub collection: String,
}
// ANCHOR_END: CollectionInsertSchema

// ANCHOR: InsertFieldSchema
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InsertFieldSchema {
    ArrayRelation {
        /// The name of the array relationship over which the related rows must be inserted
        relationship: String,
    },
    Column {
        /// The name of the column that this field should be inserted into
        column: String,
    },
    ObjectRelation {
        insertion_order: ObjectRelationInsertionOrder,
        /// The name of the object relationship over which the related row must be inserted
        relationship: String,
    },
}
// ANCHOR_END: InsertFieldSchema

// ANCHOR: ObjectRelationInsertionOrder
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ObjectRelationInsertionOrder {
    BeforeParent,
    AfterParent,
}
// ANCHOR_END: ObjectRelationInsertionOrder

// ANCHOR: MutationOperation
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MutationOperation {
    Delete {
        /// The fields to return for the rows affected by this delete operation
        returning_fields: Option<IndexMap<String, Field>>,
        /// The name of a collection
        collection: String,
        #[serde(rename = "where")]
        predicate: Option<Expression>,
    },
    Insert {
        post_insert_check: Option<Expression>,
        /// The fields to return for the rows affected by this insert operation
        returning_fields: Option<IndexMap<String, Field>>,
        /// The rows to insert into the collection
        rows: Vec<BTreeMap<String, serde_json::Value>>,
        /// The name of a collection
        collection: String,
    },
    Update {
        post_update_check: Option<Expression>,
        /// The fields to return for the rows affected by this update operation
        returning_fields: Option<IndexMap<String, Field>>,
        /// The name of a collection
        collection: String,
        /// The updates to make to the matched rows in the collection
        updates: Vec<RowUpdate>,
        #[serde(rename = "where")]
        r#where: Option<Expression>,
    },
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

// ANCHOR: RowUpdate
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RowUpdate {
    CustomOperator {
        /// The name of the column in the row
        column: String,
        operator_name: String,
        /// The value to use with the column operator
        value: serde_json::Value,
    },
    Set {
        /// The name of the column in the row
        column: String,
        /// The value to use with the column operator
        value: serde_json::Value,
    },
}
// ANCHOR_END: RowUpdate

// ANCHOR: Relationship
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Relationship {
    /// A mapping between columns on the source collection to columns on the target collection
    pub column_mapping: BTreeMap<String, String>,
    pub relationship_type: RelationshipType,
    /// The name of the collection or object type which is the source of this relationship
    pub source_collection_or_type: String,
    /// The name of a collection
    pub target_collection: String,
    /// Values to be provided to any collection arguments
    pub arguments: BTreeMap<String, RelationshipArgument>,
}
// ANCHOR_END: Relationship

// ANCHOR: RelationshipArgument
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
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
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    Object,
    Array,
}
// ANCHOR_END: RelationshipType

// ANCHOR: MutationResponse
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MutationResponse {
    /// The results of each mutation operation, in the same order as they were received
    pub operation_results: Vec<MutationOperationResults>,
}
// ANCHOR_END: MutationResponse

// ANCHOR: MutationOperationResults
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MutationOperationResults {
    /// The number of rows affected by the mutation operation
    pub affected_rows: u32,
    /// The rows affected by the mutation operation
    pub returning: Option<Vec<IndexMap<String, RowFieldValue>>>,
}
// ANCHOR_END: MutationOperationResults

#[cfg(test)]
mod tests {
    use crate::models::{self};
    use goldenfile::Mint;
    use schemars::schema_for;
    use std::{io::Write, path::PathBuf};

    #[test]
    fn test_query_request_schema() {
        let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

        let mut mint = Mint::new(&test_dir);

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
