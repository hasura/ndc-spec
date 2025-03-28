use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::BTreeMap;

use crate::{
    AggregateFunctionName, Argument, ArgumentName, ComparisonOperatorName, ExtractionFunctionName,
    FieldName, OrderDirection, PathElement, UnaryComparisonOperator, VariableName,
};

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

// ANCHOR: Grouping
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Grouping")]
pub struct Grouping {
    /// Dimensions along which to partition the data
    pub dimensions: Vec<Dimension>,
    /// Aggregates to compute in each group
    pub aggregates: IndexMap<FieldName, Aggregate>,
    /// Optionally specify a predicate to apply after grouping rows.
    /// Only used if the 'query.aggregates.group_by.filter' capability is supported.
    pub predicate: Option<GroupExpression>,
    /// Optionally specify how groups should be ordered
    /// Only used if the 'query.aggregates.group_by.order' capability is supported.
    pub order_by: Option<GroupOrderBy>,
    /// Optionally limit to N groups
    /// Only used if the 'query.aggregates.group_by.paginate' capability is supported.
    pub limit: Option<u32>,
    /// Optionally offset from the Nth group
    /// Only used if the 'query.aggregates.group_by.paginate' capability is supported.
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
    /// A scalar value to compare against
    Scalar { value: serde_json::Value },
    /// A value to compare against that is to be drawn from the query's variables.
    /// Only used if the 'query.variables' capability is supported.
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
        /// Any (object) relationships to traverse to reach this column.
        /// Only non-empty if the 'relationships' capability is supported.
        path: Vec<PathElement>,
        /// The name of the column
        column_name: FieldName,
        /// Arguments to satisfy the column specified by 'column_name'
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
        /// Path to a nested field within an object column
        field_path: Option<Vec<FieldName>>,
        /// The name of the extraction function to apply to the selected value, if any
        extraction: Option<ExtractionFunctionName>,
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
