use std::{borrow::Borrow, collections::BTreeMap};

use indexmap::IndexMap;
use ref_cast::RefCast;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use smol_str::SmolStr;

pub const VERSION_HEADER_NAME: &str = "X-Hasura-NDC-Version";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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
    pub version: String,
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
    pub mutation: MutationCapabilities,
    pub relationships: Option<RelationshipCapabilities>,
}
// ANCHOR_END: Capabilities

// ANCHOR: QueryCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Query Capabilities")]
pub struct QueryCapabilities {
    /// Does the connector support aggregate queries
    pub aggregates: Option<AggregateCapabilities>,
    /// Does the connector support queries which use variables
    pub variables: Option<LeafCapability>,
    /// Does the connector support explaining queries
    pub explain: Option<LeafCapability>,
    /// Does the connector support nested fields
    #[serde(default)]
    pub nested_fields: NestedFieldCapabilities,
    /// Does the connector support EXISTS predicates
    #[serde(default)]
    pub exists: ExistsCapabilities,
}
// ANCHOR_END: QueryCapabilities

// ANCHOR: ExistsCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Exists Capabilities")]
pub struct ExistsCapabilities {
    /// Does the connector support named scopes in column references inside
    /// EXISTS predicates
    pub named_scopes: Option<LeafCapability>,
    /// Does the connector support ExistsInCollection::Unrelated
    pub unrelated: Option<LeafCapability>,
    /// Does the connector support ExistsInCollection::NestedCollection
    pub nested_collections: Option<LeafCapability>,
    /// Does the connector support filtering over nested scalar arrays using existential quantification.
    /// This means the connector must support ExistsInCollection::NestedScalarCollection.
    pub nested_scalar_collections: Option<LeafCapability>,
}
// ANCHOR_END: ExistsCapabilities

// ANCHOR: NestedFieldCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Nested Field Capabilities")]
pub struct NestedFieldCapabilities {
    /// Does the connector support filtering by values of nested fields
    pub filter_by: Option<NestedFieldFilterByCapabilities>,
    /// Does the connector support ordering by values of nested fields
    pub order_by: Option<LeafCapability>,
    /// Does the connector support aggregating values within nested fields
    pub aggregates: Option<LeafCapability>,
    /// Does the connector support nested collection queries using
    /// `NestedField::NestedCollection`
    pub nested_collections: Option<LeafCapability>,
}
// ANCHOR_END: NestedFieldCapabilities

// ANCHOR: NestedFieldFilterByCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Nested Field Filter By Capabilities")]
pub struct NestedFieldFilterByCapabilities {
    /// Does the connector support filtering over nested arrays (ie. Expression::ArrayComparison)
    pub nested_arrays: Option<NestedArrayFilterByCapabilities>,
}
// ANCHOR_END: NestedFieldFilterByCapabilities

// ANCHOR: NestedArrayFilterByCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Nested Array Filter By Capabilities")]
pub struct NestedArrayFilterByCapabilities {
    /// Does the connector support filtering over nested arrays by checking if the array contains a value.
    /// This must be supported for all types that can be contained in an array that implement an 'eq'
    /// comparison operator.
    pub contains: Option<LeafCapability>,
    /// Does the connector support filtering over nested arrays by checking if the array is empty.
    /// This must be supported no matter what type is contained in the array.
    pub is_empty: Option<LeafCapability>,
}
// ANCHOR_END: NestedArrayFilterByCapabilities

// ANCHOR: AggregateCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Aggregate Capabilities")]
pub struct AggregateCapabilities {
    /// Does the connector support filtering based on aggregated values
    pub filter_by: Option<LeafCapability>,
    /// Does the connector support aggregations over groups
    pub group_by: Option<GroupByCapabilities>,
}
// ANCHOR_END: AggregateCapabilities

// ANCHOR: GroupByCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Group By Capabilities")]
pub struct GroupByCapabilities {
    /// Does the connector support post-grouping predicates
    pub filter: Option<LeafCapability>,
    /// Does the connector support post-grouping ordering
    pub order: Option<LeafCapability>,
    /// Does the connector support post-grouping pagination
    pub paginate: Option<LeafCapability>,
}
// ANCHOR_END: GroupByCapabilities

// ANCHOR: MutationCapabilities
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Mutation Capabilities")]
pub struct MutationCapabilities {
    /// Does the connector support executing multiple mutations in a transaction.
    pub transactional: Option<LeafCapability>,
    /// Does the connector support explaining mutations
    pub explain: Option<LeafCapability>,
}
// ANCHOR_END: MutationCapabilities

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
    pub scalar_types: BTreeMap<ScalarTypeName, ScalarType>,
    /// A list of object types which can be used as the types of arguments, or return types of procedures.
    /// Names should not overlap with scalar type names.
    pub object_types: BTreeMap<ObjectTypeName, ObjectType>,
    /// Collections which are available for queries
    pub collections: Vec<CollectionInfo>,
    /// Functions (i.e. collections which return a single column and row)
    pub functions: Vec<FunctionInfo>,
    /// Procedures which are available for execution as part of mutations
    pub procedures: Vec<ProcedureInfo>,
    /// Schema data which is relevant to features enabled by capabilities
    pub capabilities: Option<CapabilitySchemaInfo>,
}
// ANCHOR_END: SchemaResponse

// ANCHOR: ScalarType
/// The definition of a scalar type, i.e. types that can be used as the types of columns.
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Scalar Type")]
pub struct ScalarType {
    /// A description of valid values for this scalar type.
    pub representation: TypeRepresentation,
    /// A map from aggregate function names to their definitions. Result type names must be defined scalar types declared in ScalarTypesCapabilities.
    pub aggregate_functions: BTreeMap<AggregateFunctionName, AggregateFunctionDefinition>,
    /// A map from comparison operator names to their definitions. Argument type names must be defined scalar types declared in ScalarTypesCapabilities.
    pub comparison_operators: BTreeMap<ComparisonOperatorName, ComparisonOperatorDefinition>,
}
// ANCHOR_END: ScalarType

// ANCHOR: TypeRepresentation
/// Representations of scalar types
#[derive(
    Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Type Representation")]
pub enum TypeRepresentation {
    /// JSON booleans
    Boolean,
    /// Any JSON string
    String,
    /// A 8-bit signed integer with a minimum value of -2^7 and a maximum value of 2^7 - 1
    Int8,
    /// A 16-bit signed integer with a minimum value of -2^15 and a maximum value of 2^15 - 1
    Int16,
    /// A 32-bit signed integer with a minimum value of -2^31 and a maximum value of 2^31 - 1
    Int32,
    /// A 64-bit signed integer with a minimum value of -2^63 and a maximum value of 2^63 - 1
    Int64,
    /// An IEEE-754 single-precision floating-point number
    Float32,
    /// An IEEE-754 double-precision floating-point number
    Float64,
    /// Arbitrary-precision integer string
    #[serde(rename = "biginteger")]
    BigInteger,
    /// Arbitrary-precision decimal string
    #[serde(rename = "bigdecimal")]
    BigDecimal,
    /// UUID string (8-4-4-4-12)
    #[serde(rename = "uuid")]
    UUID,
    /// ISO 8601 date
    Date,
    /// ISO 8601 timestamp
    Timestamp,
    /// ISO 8601 timestamp-with-timezone
    #[serde(rename = "timestamptz")]
    TimestampTZ,
    /// GeoJSON, per RFC 7946
    Geography,
    /// GeoJSON Geometry object, per RFC 7946
    Geometry,
    /// Base64-encoded bytes
    Bytes,
    /// Arbitrary JSON
    #[serde(rename = "json")]
    JSON,
    /// One of the specified string values
    Enum { one_of: Vec<String> },
}
// ANCHOR_END: TypeRepresentation

// ANCHOR: ObjectType
/// The definition of an object type
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Object Type")]
pub struct ObjectType {
    /// Description of this type
    pub description: Option<String>,
    /// Fields defined on this object type
    pub fields: BTreeMap<FieldName, ObjectField>,
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
    /// The arguments available to the field - Matches implementation from CollectionInfo
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub arguments: BTreeMap<ArgumentName, ArgumentInfo>,
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
        /// The name can refer to a scalar or object type
        name: TypeName,
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
    /// A predicate type for a given object type
    Predicate {
        /// The object type name
        object_type_name: ObjectTypeName,
    },
}
// ANCHOR_END: Type

// ANCHOR: ComparisonOperatorDefinition
/// The definition of a comparison operator on a scalar type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Comparison Operator Definition")]
pub enum ComparisonOperatorDefinition {
    Equal,
    In,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Custom {
        /// The type of the argument to this operator
        argument_type: Type,
    },
}
// ANCHOR_END: ComparisonOperatorDefinition

// ANCHOR: AggregateFunctionDefinition
/// The definition of an aggregation function on a scalar type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Aggregate Function Definition")]
pub enum AggregateFunctionDefinition {
    Min,
    Max,
    Sum {
        /// The scalar type of the result of this function, which should have
        /// one of the type representations Int64 or Float64, depending on
        /// whether this function is defined on a scalar type with an integer or
        /// floating-point representation, respectively.
        result_type: ScalarTypeName,
    },
    Average {
        /// The scalar type of the result of this function, which should have
        /// the type representation Float64
        result_type: ScalarTypeName,
    },
    Custom {
        /// The scalar or object type of the result of this function
        result_type: Type,
    },
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
    pub name: CollectionName,
    /// Description of the collection
    pub description: Option<String>,
    /// Any arguments that this collection requires
    pub arguments: BTreeMap<ArgumentName, ArgumentInfo>,
    /// The name of the collection's object type
    #[serde(rename = "type")]
    pub collection_type: ObjectTypeName,
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
    pub name: FunctionName,
    /// Description of the function
    pub description: Option<String>,
    /// Any arguments that this collection requires
    pub arguments: BTreeMap<ArgumentName, ArgumentInfo>,
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
    pub unique_columns: Vec<FieldName>,
}
// ANCHOR_END: UniquenessConstraint

// ANCHOR: ForeignKeyConstraint
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Foreign Key Constraint")]
pub struct ForeignKeyConstraint {
    /// The columns on which you want want to define the foreign key.
    pub column_mapping: BTreeMap<FieldName, FieldName>,
    /// The name of a collection
    pub foreign_collection: CollectionName,
}
// ANCHOR_END: ForeignKeyConstraint

// ANCHOR: ProcedureInfo
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Procedure Info")]
pub struct ProcedureInfo {
    /// The name of the procedure
    pub name: ProcedureName,
    /// Column description
    pub description: Option<String>,
    /// Any arguments that this collection requires
    pub arguments: BTreeMap<ArgumentName, ArgumentInfo>,
    /// The name of the result type
    pub result_type: Type,
}
// ANCHOR_END: ProcedureInfo

// ANCHOR: CapabilitySchemaInfo
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Capability Schema Info")]
pub struct CapabilitySchemaInfo {
    /// Schema information relevant to query capabilities
    pub query: Option<QueryCapabilitiesSchemaInfo>,
}
// ANCHOR_END: CapabilitySchemaInfo

// ANCHOR: QueryCapabilitiesSchemaInfo
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Query Capabilities Schema Info")]
pub struct QueryCapabilitiesSchemaInfo {
    /// Schema information relevant to aggregate query capabilities
    pub aggregates: Option<AggregateCapabilitiesSchemaInfo>,
}
// ANCHOR_END: QueryCapabilitiesSchemaInfo

// ANCHOR: AggregateCapabilitiesSchemaInfo
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Aggregate Capabilities Schema Info")]
pub struct AggregateCapabilitiesSchemaInfo {
    /// Schema information relevant to the aggregates.filter_by capability
    pub filter_by: Option<AggregateFilterByCapabilitiesSchemaInfo>,
}
// ANCHOR_END: AggregateCapabilitiesSchemaInfo

// ANCHOR: AggregateFilterByCapabilitiesSchemaInfo
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Aggregate Filter By Capabilities Schema Info")]
pub struct AggregateFilterByCapabilitiesSchemaInfo {
    /// The scalar type which should be used for the return type of count
    /// (star_count and column_count) operations.
    pub count_scalar_type: String,
}
// ANCHOR_END: AggregateFilterByCapabilitiesSchemaInfo

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

// ANCHOR: Grouping
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Grouping")]
pub struct Grouping {
    /// Dimensions along which to partition the data
    pub dimensions: Vec<Dimension>,
    /// Aggregates to compute in each group
    pub aggregates: IndexMap<String, Aggregate>,
    /// Optionally specify a predicate to apply after grouping rows
    pub predicate: Option<GroupExpression>,
    /// Optionally specify how groups should be ordered
    pub order_by: Option<GroupOrderBy>,
    /// Optionally limit to N groups
    pub limit: Option<u32>,
    /// Optionally offset from the Nth group
    pub offset: Option<u32>,
}
// ANCHOR_END: Grouping

// ANCHOR: GroupExpression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Group Expression")]
pub enum GroupExpression {
    And {
        expressions: Vec<GroupExpression>,
    },
    Or {
        expressions: Vec<GroupExpression>,
    },
    Not {
        expression: Box<GroupExpression>,
    },
    UnaryComparisonOperator {
        target: GroupComparisonTarget,
        operator: UnaryComparisonOperator,
    },
    BinaryComparisonOperator {
        target: GroupComparisonTarget,
        operator: ComparisonOperatorName,
        value: GroupComparisonValue,
    },
}
// ANCHOR_END: GroupExpression

// ANCHOR: GroupComparisonTarget
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Aggregate Comparison Target")]
pub enum GroupComparisonTarget {
    Aggregate { aggregate: Aggregate },
}
// ANCHOR_END: GroupComparisonTarget

// ANCHOR: GroupComparisonValue
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Aggregate Comparison Value")]
pub enum GroupComparisonValue {
    Scalar { value: serde_json::Value },
    Variable { name: VariableName },
}
// ANCHOR_END: GroupComparisonValue

// ANCHOR: Dimension
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[skip_serializing_none]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Dimension")]
pub enum Dimension {
    Column {
        /// The name of the column
        column_name: FieldName,
        /// Arguments to satisfy the column specified by 'column_name'
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
        /// Path to a nested field within an object column
        field_path: Option<Vec<FieldName>>,
        /// Any (object) relationships to traverse to reach this column
        path: Vec<PathElement>,
    },
}
// ANCHOR_END: Dimension

// ANCHOR: GroupOrderBy
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Group Order By")]
pub struct GroupOrderBy {
    /// The elements to order by, in priority order
    pub elements: Vec<GroupOrderByElement>,
}
// ANCHOR_END: GroupOrderBy

// ANCHOR: GroupOrderByElement
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Group Order By Element")]
pub struct GroupOrderByElement {
    pub order_direction: OrderDirection,
    pub target: GroupOrderByTarget,
}
// ANCHOR_END: GroupOrderByElement

// ANCHOR: GroupOrderByTarget
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Group Order By Target")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GroupOrderByTarget {
    Dimension {
        /// The index of the dimension to order by, selected from the
        /// dimensions provided in the `Grouping` request.
        index: usize,
    },
    Aggregate {
        /// Aggregation method to apply
        aggregate: Aggregate,
    },
}
// ANCHOR_END: GroupOrderByTarget

// ANCHOR: Aggregate
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[skip_serializing_none]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Aggregate")]
pub enum Aggregate {
    ColumnCount {
        /// The column to apply the count aggregate function to
        column: FieldName,
        /// Arguments to satisfy the column specified by 'column'
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
        /// Path to a nested field within an object column
        field_path: Option<Vec<FieldName>>,
        /// Whether or not only distinct items should be counted
        distinct: bool,
    },
    SingleColumn {
        /// The column to apply the aggregation function to
        column: FieldName,
        /// Arguments to satisfy the column specified by 'column'
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
        /// Path to a nested field within an object column
        field_path: Option<Vec<FieldName>>,
        /// Single column aggregate function name.
        function: AggregateFunctionName,
    },
    StarCount {},
}
// ANCHOR_END: Aggregate

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
        name: FieldName,
        /// Arguments to satisfy the column specified by 'name'
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
        /// Path to a nested field within an object column
        field_path: Option<Vec<FieldName>>,
        /// Any (object) relationships to traverse to reach this column
        path: Vec<PathElement>,
    },
    Aggregate {
        /// The aggregation method to use
        aggregate: Aggregate,
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

// ANCHOR: PathElement
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(title = "Path Element")]
pub struct PathElement {
    /// The name of the relationship to follow
    pub relationship: RelationshipName,
    /// Values to be provided to any collection arguments
    pub arguments: BTreeMap<ArgumentName, RelationshipArgument>,
    /// A predicate expression to apply to the target collection
    pub predicate: Option<Box<Expression>>,
}
// ANCHOR_END: PathElement

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

// ANCHOR: Relationship
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Relationship")]
pub struct Relationship {
    /// A mapping between columns on the source collection to columns on the target collection
    pub column_mapping: BTreeMap<FieldName, FieldName>,
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

macro_rules! newtype {
    ($name: ident over $oldtype: ident) => {
        #[derive(
            Clone,
            Debug,
            Default,
            Hash,
            Eq,
            Ord,
            PartialEq,
            PartialOrd,
            Serialize,
            Deserialize,
            RefCast,
        )]
        #[repr(transparent)]
        pub struct $name($oldtype);

        impl JsonSchema for $name {
            fn schema_name() -> String {
                String::schema_name()
            }

            fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
                String::json_schema(gen)
            }

            fn is_referenceable() -> bool {
                String::is_referenceable()
            }

            fn schema_id() -> std::borrow::Cow<'static, str> {
                String::schema_id()
            }
        }

        impl AsRef<$oldtype> for $name {
            fn as_ref(&self) -> &$oldtype {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                $name(value.into())
            }
        }

        impl From<$oldtype> for $name {
            fn from(value: $oldtype) -> Self {
                $name(value)
            }
        }

        impl From<$name> for $oldtype {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl Borrow<str> for $name {
            fn borrow(&self) -> &str {
                self.0.as_str()
            }
        }

        impl Borrow<$oldtype> for $name {
            fn borrow(&self) -> &$oldtype {
                &self.0
            }
        }

        impl $name {
            pub fn new(value: $oldtype) -> Self {
                $name(value)
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            pub fn into_inner(self) -> $oldtype {
                self.0
            }

            pub fn inner(&self) -> &$oldtype {
                &self.0
            }
        }
    };
    ($name: ident) => {
        newtype! {$name over SmolStr}

        impl From<String> for $name {
            fn from(value: String) -> Self {
                $name(value.into())
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0.into()
            }
        }
    };
}

newtype! {AggregateFunctionName}
newtype! {ArgumentName}
newtype! {CollectionName}
newtype! {ComparisonOperatorName}
newtype! {FieldName}
newtype! {FunctionName over CollectionName}
newtype! {ObjectTypeName over TypeName}
newtype! {ProcedureName}
newtype! {RelationshipName}
newtype! {ScalarTypeName over TypeName}
newtype! {TypeName}
newtype! {VariableName}

impl From<String> for FunctionName {
    fn from(value: String) -> Self {
        FunctionName(value.into())
    }
}

impl From<FunctionName> for String {
    fn from(value: FunctionName) -> Self {
        value.0.into()
    }
}

impl From<String> for ObjectTypeName {
    fn from(value: String) -> Self {
        ObjectTypeName(value.into())
    }
}

impl From<ObjectTypeName> for String {
    fn from(value: ObjectTypeName) -> Self {
        value.0.into()
    }
}

impl From<String> for ScalarTypeName {
    fn from(value: String) -> Self {
        ScalarTypeName(value.into())
    }
}

impl From<ScalarTypeName> for String {
    fn from(value: ScalarTypeName) -> Self {
        value.0.into()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::path::PathBuf;

    use goldenfile::Mint;
    use schemars::schema_for;

    use super::*;

    #[test]
    fn test_json_schemas() {
        let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

        let mut mint = Mint::new(test_dir);

        test_json_schema(
            &mut mint,
            &schema_for!(ErrorResponse),
            "error_response.jsonschema",
        );

        test_json_schema(
            &mut mint,
            &schema_for!(SchemaResponse),
            "schema_response.jsonschema",
        );

        test_json_schema(
            &mut mint,
            &schema_for!(CapabilitiesResponse),
            "capabilities_response.jsonschema",
        );

        test_json_schema(
            &mut mint,
            &schema_for!(QueryRequest),
            "query_request.jsonschema",
        );
        test_json_schema(
            &mut mint,
            &schema_for!(QueryResponse),
            "query_response.jsonschema",
        );

        test_json_schema(
            &mut mint,
            &schema_for!(ExplainResponse),
            "explain_response.jsonschema",
        );

        test_json_schema(
            &mut mint,
            &schema_for!(MutationRequest),
            "mutation_request.jsonschema",
        );
        test_json_schema(
            &mut mint,
            &schema_for!(MutationResponse),
            "mutation_response.jsonschema",
        );
    }

    fn test_json_schema(mint: &mut Mint, schema: &schemars::schema::RootSchema, filename: &str) {
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
