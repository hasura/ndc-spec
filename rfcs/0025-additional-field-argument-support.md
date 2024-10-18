# Additional Field Argument Support

## Problem

Object type fields can declare arguments that must be submitted when the field is evaluated. However, support for using these fields is not universal; there are some features which do not allow the use of fields with arguments, for example in nested field paths, or in relationship column mappings. Those particular examples are harder to address, but there are a few low hanging places where we could easily add support for field arguments in a non-breaking way.

These places are:

- `ComparisonTarget::Column` - one cannot compare _against_ column (ie LHS) that takes arguments
- `ComparisonValue::Column` - one cannot compare _to_ a column (ie. RHS) that takes arguments
- `OrderByTarget::Column` - one cannot order by columns that take arguments
- `Aggregate::ColumnCount` - one cannot count by a column that takes arguments
- `Aggregate::SingleColumn` - one cannot aggregate a single column that takes arguments

## Solution

Simply adding:

```rust
#[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
arguments: BTreeMap<ArgumentName, Argument>,
```

to all these enum variants resolves the problem in a non-breaking manner.
