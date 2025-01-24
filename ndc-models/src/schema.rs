use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeMap;

use crate::{
    AggregateFunctionName, ArgumentName, CollectionName, ComparisonOperatorName,
    ExtractionFunctionName, FieldName, FunctionName, ObjectTypeName, ProcedureName, ScalarTypeName,
    TypeName,
};

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
    /// A map from extraction function names to their defginitions.
    pub extraction_functions: BTreeMap<ExtractionFunctionName, ExtractionFunctionDefinition>,
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
    /// Any foreign keys defined for this object type's columns
    pub foreign_keys: BTreeMap<String, ForeignKeyConstraint>,
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
    Contains,
    ContainsInsensitive,
    StartsWith,
    StartsWithInsensitive,
    EndsWith,
    EndsWithInsensitive,
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

// ANCHOR: ExtractionFunctionDefinition
/// The definition of an aggregation function on a scalar type
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Extraction Function Definition")]
pub struct ExtractionFunctionDefinition {
    /// The result type, which must be a defined scalar types in the schema response.
    result_type: ScalarTypeName,
    /// The meaning of this extraction function
    r#type: ExtractionFunctionType,
}
// ANCHOR_END: ExtractionFunctionDefinition

// ANCHOR: ExtractionFunctionType
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(title = "Extraction Function Definition")]
pub enum ExtractionFunctionType {
    Nanosecond,
    Microsecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
    DayOfWeek,
    DayOfYear,
    Custom,
}
// ANCHOR_END: ExtractionFunctionType

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
    /// This is a mapping between fields on object type to columns on the foreign collection.
    /// The column on the foreign collection is specified via a field path (ie. an array of field
    /// names that descend through nested object fields). The field path must only contain a single item,
    /// meaning a column on the foreign collection's type, unless the 'relationships.nested'
    /// capability is supported, in which case multiple items can be used to denote a nested object field.
    pub column_mapping: BTreeMap<FieldName, Vec<FieldName>>,
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
    /// The scalar type which should be used for the return type of count
    /// (star_count and column_count) operations.
    pub count_scalar_type: ScalarTypeName,
}
// ANCHOR_END: AggregateCapabilitiesSchemaInfo
