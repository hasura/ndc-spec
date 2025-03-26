use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{CastType, RelationalLiteral, Sort};
use crate::{FieldName, ScopeName};

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[schemars(title = "RelationalExpression")]
pub enum RelationalExpression {
    // Data selection
    Literal {
        literal: RelationalLiteral,
    },
    Column {
        scope: ScopeName,
        name: FieldName,
    },

    // Conditional operators
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.conditional.case`
    /// * During filtering: `relational_query.filter.conditional.case`
    /// * During comparison:`relational_query.sort.expression.conditional.case`
    /// * During joining: `relational_query.join.expression.conditional.case`
    /// * During aggregation: `relational_query.aggregate.expression.conditional.case`
    /// * During windowing: `relational_query.window.expression.conditional.case`
    Case {
        when: Vec<CaseWhen>,
        default: Option<Box<RelationalExpression>>,
    },

    // Logical operators
    And {
        left: Box<RelationalExpression>,
        right: Box<RelationalExpression>,
    },
    Or {
        left: Box<RelationalExpression>,
        right: Box<RelationalExpression>,
    },
    Not {
        expr: Box<RelationalExpression>,
    },

    // Comparison operators
    Eq {
        left: Box<RelationalExpression>,
        right: Box<RelationalExpression>,
    },
    NotEq {
        left: Box<RelationalExpression>,
        right: Box<RelationalExpression>,
    },
    Lt {
        left: Box<RelationalExpression>,
        right: Box<RelationalExpression>,
    },
    LtEq {
        left: Box<RelationalExpression>,
        right: Box<RelationalExpression>,
    },
    Gt {
        left: Box<RelationalExpression>,
        right: Box<RelationalExpression>,
    },
    GtEq {
        left: Box<RelationalExpression>,
        right: Box<RelationalExpression>,
    },
    IsNotNull {
        expr: Box<RelationalExpression>,
    },
    IsNull {
        expr: Box<RelationalExpression>,
    },
    IsTrue {
        expr: Box<RelationalExpression>,
    },
    IsFalse {
        expr: Box<RelationalExpression>,
    },
    IsNotTrue {
        expr: Box<RelationalExpression>,
    },
    IsNotFalse {
        expr: Box<RelationalExpression>,
    },
    In {
        expr: Box<RelationalExpression>,
        list: Vec<RelationalExpression>,
    },
    NotIn {
        expr: Box<RelationalExpression>,
        list: Vec<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.comparison.like`
    /// * During filtering: `relational_query.filter.comparison.like`
    /// * During comparison:`relational_query.sort.expression.comparison.like`
    /// * During joining: `relational_query.join.expression.comparison.like`
    /// * During aggregation: `relational_query.aggregate.expression.comparison.like`
    /// * During windowing: `relational_query.window.expression.comparison.like`
    Like {
        expr: Box<RelationalExpression>,
        pattern: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.comparison.like`
    /// * During filtering: `relational_query.filter.comparison.like`
    /// * During comparison:`relational_query.sort.expression.comparison.like`
    /// * During joining: `relational_query.join.expression.comparison.like`
    /// * During aggregation: `relational_query.aggregate.expression.comparison.like`
    /// * During windowing: `relational_query.window.expression.comparison.like`
    NotLike {
        expr: Box<RelationalExpression>,
        pattern: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.comparison.ilike`
    /// * During filtering: `relational_query.filter.comparison.ilike`
    /// * During comparison:`relational_query.sort.expression.comparison.ilike`
    /// * During joining: `relational_query.join.expression.comparison.ilike`
    /// * During aggregation: `relational_query.aggregate.expression.comparison.ilike`
    /// * During windowing: `relational_query.window.expression.comparison.ilike`
    ILike {
        expr: Box<RelationalExpression>,
        pattern: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.comparison.ilike`
    /// * During filtering: `relational_query.filter.comparison.ilike`
    /// * During comparison:`relational_query.sort.expression.comparison.ilike`
    /// * During joining: `relational_query.join.expression.comparison.ilike`
    /// * During aggregation: `relational_query.aggregate.expression.comparison.ilike`
    /// * During windowing: `relational_query.window.expression.comparison.ilike`
    NotILike {
        expr: Box<RelationalExpression>,
        pattern: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.comparison.between`
    /// * During filtering: `relational_query.filter.comparison.between`
    /// * During comparison:`relational_query.sort.expression.comparison.between`
    /// * During joining: `relational_query.join.expression.comparison.between`
    /// * During aggregation: `relational_query.aggregate.expression.comparison.between`
    /// * During windowing: `relational_query.window.expression.comparison.between`
    Between {
        low: Box<RelationalExpression>,
        expr: Box<RelationalExpression>,
        high: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.comparison.between`
    /// * During filtering: `relational_query.filter.comparison.between`
    /// * During comparison:`relational_query.sort.expression.comparison.between`
    /// * During joining: `relational_query.join.expression.comparison.between`
    /// * During aggregation: `relational_query.aggregate.expression.comparison.between`
    /// * During windowing: `relational_query.window.expression.comparison.between`
    NotBetween {
        low: Box<RelationalExpression>,
        expr: Box<RelationalExpression>,
        high: Box<RelationalExpression>,
    },

    // Arithmetic operators
    Plus {
        left: Box<RelationalExpression>,
        right: Box<RelationalExpression>,
    },
    Minus {
        left: Box<RelationalExpression>,
        right: Box<RelationalExpression>,
    },
    Multiply {
        left: Box<RelationalExpression>,
        right: Box<RelationalExpression>,
    },
    Divide {
        left: Box<RelationalExpression>,
        right: Box<RelationalExpression>,
    },
    Modulo {
        left: Box<RelationalExpression>,
        right: Box<RelationalExpression>,
    },
    Negate {
        expr: Box<RelationalExpression>,
    },

    // Scalar functions
    Cast {
        expr: Box<RelationalExpression>,
        as_type: CastType,
    },
    TryCast {
        expr: Box<RelationalExpression>,
        as_type: CastType,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.abs`
    /// * During filtering: `relational_query.filter.scalar.abs`
    /// * During comparison:`relational_query.sort.expression.scalar.abs`
    /// * During joining: `relational_query.join.expression.scalar.abs`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.abs`
    /// * During windowing: `relational_query.window.expression.scalar.abs`
    Abs {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.btrim`
    /// * During filtering: `relational_query.filter.scalar.btrim`
    /// * During comparison:`relational_query.sort.expression.scalar.btrim`
    /// * During joining: `relational_query.join.expression.scalar.btrim`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.btrim`
    /// * During windowing: `relational_query.window.expression.scalar.btrim`
    BTrim {
        str: Box<RelationalExpression>,
        trim_str: Option<Box<RelationalExpression>>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.ceil`
    /// * During filtering: `relational_query.filter.scalar.ceil`
    /// * During comparison:`relational_query.sort.expression.scalar.ceil`
    /// * During joining: `relational_query.join.expression.scalar.ceil`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.ceil`
    /// * During windowing: `relational_query.window.expression.scalar.ceil`
    Ceil {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.character_length`
    /// * During filtering: `relational_query.filter.scalar.character_length`
    /// * During comparison:`relational_query.sort.expression.scalar.character_length`
    /// * During joining: `relational_query.join.expression.scalar.character_length`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.character_length`
    /// * During windowing: `relational_query.window.expression.scalar.character_length`
    CharacterLength {
        str: Box<RelationalExpression>,
    },
    Coalesce {
        exprs: Vec<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.concat`
    /// * During filtering: `relational_query.filter.scalar.concat`
    /// * During comparison:`relational_query.sort.expression.scalar.concat`
    /// * During joining: `relational_query.join.expression.scalar.concat`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.concat`
    /// * During windowing: `relational_query.window.expression.scalar.concat`
    Concat {
        exprs: Vec<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.contains`
    /// * During filtering: `relational_query.filter.scalar.contains`
    /// * During comparison:`relational_query.sort.expression.scalar.contains`
    /// * During joining: `relational_query.join.expression.scalar.contains`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.contains`
    /// * During windowing: `relational_query.window.expression.scalar.contains`
    Contains {
        str: Box<RelationalExpression>,
        search_str: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.cos`
    /// * During filtering: `relational_query.filter.scalar.cos`
    /// * During comparison:`relational_query.sort.expression.scalar.cos`
    /// * During joining: `relational_query.join.expression.scalar.cos`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.cos`
    /// * During windowing: `relational_query.window.expression.scalar.cos`
    Cos {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.current_date`
    /// * During filtering: `relational_query.filter.scalar.current_date`
    /// * During comparison:`relational_query.sort.expression.scalar.current_date`
    /// * During joining: `relational_query.join.expression.scalar.current_date`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.current_date`
    /// * During windowing: `relational_query.window.expression.scalar.current_date`
    CurrentDate,
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.current_time`
    /// * During filtering: `relational_query.filter.scalar.current_time`
    /// * During comparison:`relational_query.sort.expression.scalar.current_time`
    /// * During joining: `relational_query.join.expression.scalar.current_time`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.current_time`
    /// * During windowing: `relational_query.window.expression.scalar.current_time`
    CurrentTime,
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.current_timestamp`
    /// * During filtering: `relational_query.filter.scalar.current_timestamp`
    /// * During comparison:`relational_query.sort.expression.scalar.current_timestamp`
    /// * During joining: `relational_query.join.expression.scalar.current_timestamp`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.current_timestamp`
    /// * During windowing: `relational_query.window.expression.scalar.current_timestamp`
    CurrentTimestamp,
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.date_part`
    /// * During filtering: `relational_query.filter.scalar.date_part`
    /// * During comparison:`relational_query.sort.expression.scalar.date_part`
    /// * During joining: `relational_query.join.expression.scalar.date_part`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.date_part`
    /// * During windowing: `relational_query.window.expression.scalar.date_part`
    DatePart {
        expr: Box<RelationalExpression>,
        part: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.date_trunc`
    /// * During filtering: `relational_query.filter.scalar.date_trunc`
    /// * During comparison:`relational_query.sort.expression.scalar.date_trunc`
    /// * During joining: `relational_query.join.expression.scalar.date_trunc`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.date_trunc`
    /// * During windowing: `relational_query.window.expression.scalar.date_trunc`
    DateTrunc {
        expr: Box<RelationalExpression>,
        part: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.exp`
    /// * During filtering: `relational_query.filter.scalar.exp`
    /// * During comparison:`relational_query.sort.expression.scalar.exp`
    /// * During joining: `relational_query.join.expression.scalar.exp`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.exp`
    /// * During windowing: `relational_query.window.expression.scalar.exp`
    Exp {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.floor`
    /// * During filtering: `relational_query.filter.scalar.floor`
    /// * During comparison:`relational_query.sort.expression.scalar.floor`
    /// * During joining: `relational_query.join.expression.scalar.floor`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.floor`
    /// * During windowing: `relational_query.window.expression.scalar.floor`
    Floor {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.greatest`
    /// * During filtering: `relational_query.filter.scalar.greatest`
    /// * During comparison:`relational_query.sort.expression.scalar.greatest`
    /// * During joining: `relational_query.join.expression.scalar.greatest`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.greatest`
    /// * During windowing: `relational_query.window.expression.scalar.greatest`
    Greatest {
        exprs: Vec<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.is_nan`
    /// * During filtering: `relational_query.filter.scalar.is_nan`
    /// * During comparison:`relational_query.sort.expression.scalar.is_nan`
    /// * During joining: `relational_query.join.expression.scalar.is_nan`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.is_nan`
    /// * During windowing: `relational_query.window.expression.scalar.is_nan`
    IsNaN {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.is_zero`
    /// * During filtering: `relational_query.filter.scalar.is_zero`
    /// * During comparison:`relational_query.sort.expression.scalar.is_zero`
    /// * During joining: `relational_query.join.expression.scalar.is_zero`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.is_zero`
    /// * During windowing: `relational_query.window.expression.scalar.is_zero`
    IsZero {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.least`
    /// * During filtering: `relational_query.filter.scalar.least`
    /// * During comparison:`relational_query.sort.expression.scalar.least`
    /// * During joining: `relational_query.join.expression.scalar.least`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.least`
    /// * During windowing: `relational_query.window.expression.scalar.least`
    Least {
        exprs: Vec<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.left`
    /// * During filtering: `relational_query.filter.scalar.left`
    /// * During comparison:`relational_query.sort.expression.scalar.left`
    /// * During joining: `relational_query.join.expression.scalar.left`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.left`
    /// * During windowing: `relational_query.window.expression.scalar.left`
    Left {
        str: Box<RelationalExpression>,
        n: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.ln`
    /// * During filtering: `relational_query.filter.scalar.ln`
    /// * During comparison:`relational_query.sort.expression.scalar.ln`
    /// * During joining: `relational_query.join.expression.scalar.ln`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.ln`
    /// * During windowing: `relational_query.window.expression.scalar.ln`
    Ln {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.log`
    /// * During filtering: `relational_query.filter.scalar.log`
    /// * During comparison:`relational_query.sort.expression.scalar.log`
    /// * During joining: `relational_query.join.expression.scalar.log`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.log`
    /// * During windowing: `relational_query.window.expression.scalar.log`
    Log {
        expr: Box<RelationalExpression>,
        base: Option<Box<RelationalExpression>>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.log10`
    /// * During filtering: `relational_query.filter.scalar.log10`
    /// * During comparison:`relational_query.sort.expression.scalar.log10`
    /// * During joining: `relational_query.join.expression.scalar.log10`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.log10`
    /// * During windowing: `relational_query.window.expression.scalar.log10`
    Log10 {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.log2`
    /// * During filtering: `relational_query.filter.scalar.log2`
    /// * During comparison:`relational_query.sort.expression.scalar.log2`
    /// * During joining: `relational_query.join.expression.scalar.log2`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.log2`
    /// * During windowing: `relational_query.window.expression.scalar.log2`
    Log2 {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.lpad`
    /// * During filtering: `relational_query.filter.scalar.lpad`
    /// * During comparison:`relational_query.sort.expression.scalar.lpad`
    /// * During joining: `relational_query.join.expression.scalar.lpad`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.lpad`
    /// * During windowing: `relational_query.window.expression.scalar.lpad`
    LPad {
        str: Box<RelationalExpression>,
        n: Box<RelationalExpression>,
        padding_str: Option<Box<RelationalExpression>>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.ltrim`
    /// * During filtering: `relational_query.filter.scalar.ltrim`
    /// * During comparison:`relational_query.sort.expression.scalar.ltrim`
    /// * During joining: `relational_query.join.expression.scalar.ltrim`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.ltrim`
    /// * During windowing: `relational_query.window.expression.scalar.ltrim`
    LTrim {
        str: Box<RelationalExpression>,
        trim_str: Option<Box<RelationalExpression>>,
    },
    NullIf {
        expr1: Box<RelationalExpression>,
        expr2: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.nvl`
    /// * During filtering: `relational_query.filter.scalar.nvl`
    /// * During comparison:`relational_query.sort.expression.scalar.nvl`
    /// * During joining: `relational_query.join.expression.scalar.nvl`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.nvl`
    /// * During windowing: `relational_query.window.expression.scalar.nvl`
    Nvl {
        expr1: Box<RelationalExpression>,
        expr2: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.power`
    /// * During filtering: `relational_query.filter.scalar.power`
    /// * During comparison:`relational_query.sort.expression.scalar.power`
    /// * During joining: `relational_query.join.expression.scalar.power`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.power`
    /// * During windowing: `relational_query.window.expression.scalar.power`
    Power {
        base: Box<RelationalExpression>,
        exp: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.random`
    /// * During filtering: `relational_query.filter.scalar.random`
    /// * During comparison:`relational_query.sort.expression.scalar.random`
    /// * During joining: `relational_query.join.expression.scalar.random`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.random`
    /// * During windowing: `relational_query.window.expression.scalar.random`
    Random,
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.replace`
    /// * During filtering: `relational_query.filter.scalar.replace`
    /// * During comparison:`relational_query.sort.expression.scalar.replace`
    /// * During joining: `relational_query.join.expression.scalar.replace`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.replace`
    /// * During windowing: `relational_query.window.expression.scalar.replace`
    Replace {
        str: Box<RelationalExpression>,
        substr: Box<RelationalExpression>,
        replacement: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.reverse`
    /// * During filtering: `relational_query.filter.scalar.reverse`
    /// * During comparison:`relational_query.sort.expression.scalar.reverse`
    /// * During joining: `relational_query.join.expression.scalar.reverse`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.reverse`
    /// * During windowing: `relational_query.window.expression.scalar.reverse`
    Reverse {
        str: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.right`
    /// * During filtering: `relational_query.filter.scalar.right`
    /// * During comparison:`relational_query.sort.expression.scalar.right`
    /// * During joining: `relational_query.join.expression.scalar.right`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.right`
    /// * During windowing: `relational_query.window.expression.scalar.right`
    Right {
        str: Box<RelationalExpression>,
        n: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.round`
    /// * During filtering: `relational_query.filter.scalar.round`
    /// * During comparison:`relational_query.sort.expression.scalar.round`
    /// * During joining: `relational_query.join.expression.scalar.round`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.round`
    /// * During windowing: `relational_query.window.expression.scalar.round`
    Round {
        expr: Box<RelationalExpression>,
        prec: Option<Box<RelationalExpression>>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.rpad`
    /// * During filtering: `relational_query.filter.scalar.rpad`
    /// * During comparison:`relational_query.sort.expression.scalar.rpad`
    /// * During joining: `relational_query.join.expression.scalar.rpad`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.rpad`
    /// * During windowing: `relational_query.window.expression.scalar.rpad`
    RPad {
        str: Box<RelationalExpression>,
        n: Box<RelationalExpression>,
        padding_str: Option<Box<RelationalExpression>>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.rtrim`
    /// * During filtering: `relational_query.filter.scalar.rtrim`
    /// * During comparison:`relational_query.sort.expression.scalar.rtrim`
    /// * During joining: `relational_query.join.expression.scalar.rtrim`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.rtrim`
    /// * During windowing: `relational_query.window.expression.scalar.rtrim`
    RTrim {
        str: Box<RelationalExpression>,
        trim_str: Option<Box<RelationalExpression>>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.sqrt`
    /// * During filtering: `relational_query.filter.scalar.sqrt`
    /// * During comparison:`relational_query.sort.expression.scalar.sqrt`
    /// * During joining: `relational_query.join.expression.scalar.sqrt`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.sqrt`
    /// * During windowing: `relational_query.window.expression.scalar.sqrt`
    Sqrt {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.strpos`
    /// * During filtering: `relational_query.filter.scalar.strpos`
    /// * During comparison:`relational_query.sort.expression.scalar.strpos`
    /// * During joining: `relational_query.join.expression.scalar.strpos`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.strpos`
    /// * During windowing: `relational_query.window.expression.scalar.strpos`
    StrPos {
        str: Box<RelationalExpression>,
        substr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.substr`
    /// * During filtering: `relational_query.filter.scalar.substr`
    /// * During comparison:`relational_query.sort.expression.scalar.substr`
    /// * During joining: `relational_query.join.expression.scalar.substr`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.substr`
    /// * During windowing: `relational_query.window.expression.scalar.substr`
    Substr {
        str: Box<RelationalExpression>,
        start_pos: Box<RelationalExpression>,
        len: Option<Box<RelationalExpression>>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.substr_index`
    /// * During filtering: `relational_query.filter.scalar.substr_index`
    /// * During comparison:`relational_query.sort.expression.scalar.substr_index`
    /// * During joining: `relational_query.join.expression.scalar.substr_index`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.substr_index`
    /// * During windowing: `relational_query.window.expression.scalar.substr_index`
    SubstrIndex {
        str: Box<RelationalExpression>,
        delim: Box<RelationalExpression>,
        count: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.tan`
    /// * During filtering: `relational_query.filter.scalar.tan`
    /// * During comparison:`relational_query.sort.expression.scalar.tan`
    /// * During joining: `relational_query.join.expression.scalar.tan`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.tan`
    /// * During windowing: `relational_query.window.expression.scalar.tan`
    Tan {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.to_date`
    /// * During filtering: `relational_query.filter.scalar.to_date`
    /// * During comparison:`relational_query.sort.expression.scalar.to_date`
    /// * During joining: `relational_query.join.expression.scalar.to_date`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.to_date`
    /// * During windowing: `relational_query.window.expression.scalar.to_date`
    ToDate {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.to_timestamp`
    /// * During filtering: `relational_query.filter.scalar.to_timestamp`
    /// * During comparison:`relational_query.sort.expression.scalar.to_timestamp`
    /// * During joining: `relational_query.join.expression.scalar.to_timestamp`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.to_timestamp`
    /// * During windowing: `relational_query.window.expression.scalar.to_timestamp`
    ToTimestamp {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.trunc`
    /// * During filtering: `relational_query.filter.scalar.trunc`
    /// * During comparison:`relational_query.sort.expression.scalar.trunc`
    /// * During joining: `relational_query.join.expression.scalar.trunc`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.trunc`
    /// * During windowing: `relational_query.window.expression.scalar.trunc`
    Trunc {
        expr: Box<RelationalExpression>,
        prec: Option<Box<RelationalExpression>>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.to_lower`
    /// * During filtering: `relational_query.filter.scalar.to_lower`
    /// * During comparison:`relational_query.sort.expression.scalar.to_lower`
    /// * During joining: `relational_query.join.expression.scalar.to_lower`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.to_lower`
    /// * During windowing: `relational_query.window.expression.scalar.to_lower`
    ToLower {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.scalar.to_upper`
    /// * During filtering: `relational_query.filter.scalar.to_upper`
    /// * During comparison:`relational_query.sort.expression.scalar.to_upper`
    /// * During joining: `relational_query.join.expression.scalar.to_upper`
    /// * During aggregation: `relational_query.aggregate.expression.scalar.to_upper`
    /// * During windowing: `relational_query.window.expression.scalar.to_upper`
    ToUpper {
        expr: Box<RelationalExpression>,
    },

    // acos
    // acosh
    // ascii
    // asin
    // asinh
    // atan
    // atan2
    // atanh
    // bit_length
    // btrim
    // cbrt
    // chr
    // coalesce
    // concat_ws
    // contains
    // cosh
    // cot
    // decode
    // degrees
    // digest
    // encode
    // ends_with
    // factorial
    // find_in_set
    // gcd
    // initcap
    // instr
    // lcm
    // levenshtein
    // make_date
    // md5
    // nanvl
    // nvl2
    // octet_length
    // pi
    // radians
    // regexp_count
    // regexp_like
    // regexp_match
    // regexp_replace
    // repeat
    // sha224
    // sha256
    // sha384
    // sha512
    // signum
    // sin
    // sinh
    // split_part
    // starts_with
    // tanh
    // to_char
    // to_hex
    // to_timestamp_micros
    // to_timestamp_millis
    // to_timestamp_nanos
    // to_timestamp_seconds
    // today
    // translate
    // uuid

    // Aggregate functions
    Average {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.aggregate.bool_and`
    /// * During filtering: `relational_query.filter.aggregate.bool_and`
    /// * During comparison:`relational_query.sort.expression.aggregate.bool_and`
    /// * During joining: `relational_query.join.expression.aggregate.bool_and`
    /// * During aggregation: `relational_query.aggregate.expression.aggregate.bool_and`
    /// * During windowing: `relational_query.window.expression.aggregate.bool_and`
    BoolAnd {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.aggregate.bool_or`
    /// * During filtering: `relational_query.filter.aggregate.bool_or`
    /// * During comparison:`relational_query.sort.expression.aggregate.bool_or`
    /// * During joining: `relational_query.join.expression.aggregate.bool_or`
    /// * During aggregation: `relational_query.aggregate.expression.aggregate.bool_or`
    /// * During windowing: `relational_query.window.expression.aggregate.bool_or`
    BoolOr {
        expr: Box<RelationalExpression>,
    },
    Count {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.aggregate.first_value`
    /// * During filtering: `relational_query.filter.aggregate.first_value`
    /// * During comparison:`relational_query.sort.expression.aggregate.first_value`
    /// * During joining: `relational_query.join.expression.aggregate.first_value`
    /// * During aggregation: `relational_query.aggregate.expression.aggregate.first_value`
    /// * During windowing: `relational_query.window.expression.aggregate.first_value`
    FirstValue {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.aggregate.last_value`
    /// * During filtering: `relational_query.filter.aggregate.last_value`
    /// * During comparison:`relational_query.sort.expression.aggregate.last_value`
    /// * During joining: `relational_query.join.expression.aggregate.last_value`
    /// * During aggregation: `relational_query.aggregate.expression.aggregate.last_value`
    /// * During windowing: `relational_query.window.expression.aggregate.last_value`
    LastValue {
        expr: Box<RelationalExpression>,
    },
    Max {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.aggregate.mean`
    /// * During filtering: `relational_query.filter.aggregate.mean`
    /// * During comparison:`relational_query.sort.expression.aggregate.mean`
    /// * During joining: `relational_query.join.expression.aggregate.mean`
    /// * During aggregation: `relational_query.aggregate.expression.aggregate.mean`
    /// * During windowing: `relational_query.window.expression.aggregate.mean`
    Mean {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.aggregate.median`
    /// * During filtering: `relational_query.filter.aggregate.median`
    /// * During comparison:`relational_query.sort.expression.aggregate.median`
    /// * During joining: `relational_query.join.expression.aggregate.median`
    /// * During aggregation: `relational_query.aggregate.expression.aggregate.median`
    /// * During windowing: `relational_query.window.expression.aggregate.median`
    Median {
        expr: Box<RelationalExpression>,
    },
    Min {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.aggregate.string_agg`
    /// * During filtering: `relational_query.filter.aggregate.string_agg`
    /// * During comparison:`relational_query.sort.expression.aggregate.string_agg`
    /// * During joining: `relational_query.join.expression.aggregate.string_agg`
    /// * During aggregation: `relational_query.aggregate.expression.aggregate.string_agg`
    /// * During windowing: `relational_query.window.expression.aggregate.string_agg`
    StringAgg {
        expr: Box<RelationalExpression>,
    },
    Sum {
        expr: Box<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.aggregate.var`
    /// * During filtering: `relational_query.filter.aggregate.var`
    /// * During comparison:`relational_query.sort.expression.aggregate.var`
    /// * During joining: `relational_query.join.expression.aggregate.var`
    /// * During aggregation: `relational_query.aggregate.expression.aggregate.var`
    /// * During windowing: `relational_query.window.expression.aggregate.var`
    Var {
        expr: Box<RelationalExpression>,
    },
    // array_agg
    // bit_and
    // bit_or
    // bit_xor
    // grouping
    // var_pop
    // var_population
    // var_samp
    // var_sample

    // Window functions
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.window.row_number`
    /// * During filtering: `relational_query.filter.window.row_number`
    /// * During comparison:`relational_query.sort.expression.window.row_number`
    /// * During joining: `relational_query.join.expression.window.row_number`
    /// * During aggregation: `relational_query.window.row_number`
    /// * During windowing: `relational_query.window.expression.window.row_number`
    RowNumber {
        order_by: Vec<Sort>,
        partition_by: Vec<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.window.dense_rank`
    /// * During filtering: `relational_query.filter.window.dense_rank`
    /// * During comparison:`relational_query.sort.expression.window.dense_rank`
    /// * During joining: `relational_query.join.expression.window.dense_rank`
    /// * During aggregation: `relational_query.window.dense_rank`
    /// * During windowing: `relational_query.window.expression.window.dense_rank`
    DenseRank {
        order_by: Vec<Sort>,
        partition_by: Vec<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.window.ntile`
    /// * During filtering: `relational_query.filter.window.ntile`
    /// * During comparison:`relational_query.sort.expression.window.ntile`
    /// * During joining: `relational_query.join.expression.window.ntile`
    /// * During aggregation: `relational_query.window.ntile`
    /// * During windowing: `relational_query.window.expression.window.ntile`
    NTile {
        order_by: Vec<Sort>,
        partition_by: Vec<RelationalExpression>,
        n: i64,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.window.rank`
    /// * During filtering: `relational_query.filter.window.rank`
    /// * During comparison:`relational_query.sort.expression.window.rank`
    /// * During joining: `relational_query.join.expression.window.rank`
    /// * During aggregation: `relational_query.window.rank`
    /// * During windowing: `relational_query.window.expression.window.rank`
    Rank {
        order_by: Vec<Sort>,
        partition_by: Vec<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.window.cume_dist`
    /// * During filtering: `relational_query.filter.window.cume_dist`
    /// * During comparison:`relational_query.sort.expression.window.cume_dist`
    /// * During joining: `relational_query.join.expression.window.cume_dist`
    /// * During aggregation: `relational_query.window.cume_dist`
    /// * During windowing: `relational_query.window.expression.window.cume_dist`
    CumeDist {
        order_by: Vec<Sort>,
        partition_by: Vec<RelationalExpression>,
    },
    /// Only used when in specific contexts where the appropriate capability is supported:
    /// * During projection: `relational_query.project.expression.window.percent_rank`
    /// * During filtering: `relational_query.filter.window.percent_rank`
    /// * During comparison:`relational_query.sort.expression.window.percent_rank`
    /// * During joining: `relational_query.join.expression.window.percent_rank`
    /// * During aggregation: `relational_query.window.percent_rank`
    /// * During windowing: `relational_query.window.expression.window.percent_rank`
    PercentRank {
        order_by: Vec<Sort>,
        partition_by: Vec<RelationalExpression>,
    },
    // lag
    // lead
    // nth_value
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "CaseWhen")]
pub struct CaseWhen {
    pub when: RelationalExpression,
    pub then: RelationalExpression,
}
