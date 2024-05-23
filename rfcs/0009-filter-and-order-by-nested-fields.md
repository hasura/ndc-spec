# Filtering and ordering by nested fields

## Purpose

Some databases, e.g. MongoDB and other document databases, allow collections to contain "columns" with nested structures (i.e. "objects") rather than just scalar values.
It would be useful to allow queries on such collections to be filtered and sorted based on the values in nested fields rather than just top-level scalar column values.

## Proposal

We modify `ComparisonTarget` and `OrderByTarget` to add a `field_path` property, as follows:

```rust
pub enum ComparisonTarget {
    Column {
        name: String,
        field_path: Option<Vec<String>>,
        path: Vec<PathElement>,
    },
    RootCollectionColumn {
        name: String,
        field_path: Option<Vec<String>>,
    },
}

pub enum OrderByTarget {
    Column {
        name: String,
        field_path: Option<Vec<String>>,
        path: Vec<PathElement>,
    },
    SingleColumnAggregate {
        column: String,
        function: String,
        path: Vec<PathElement>,
    },
    StarCountAggregate {
        path: Vec<PathElement>,
    },
}
```

When `field_path` is present and non-empty it refers to a path to a nested field within the column.
The value of the nested field will be used for comparison or ordering instead of using the full value of the column.

A connector can declare that it supports filtering and/or ordering by nested fields via two new capabilities: `query.nested_fields.filter_by` and `query.nested_fields.order_by`.
These capabilities declare whether the connector can handle non-empty `field_path`


## Future extensions

Although out of scope for this RFC, in the future we probably want to extend aggregates to allow aggregating on values of nested fields.
This could be achieved by adding a `field_path` property to `Aggregate::ColumnCount` and `Aggregate::SingleColumn`.
We could also order by aggregates on nested fields by adding `field_path` to `OrderByTarget::SingleColumnAggregate`.