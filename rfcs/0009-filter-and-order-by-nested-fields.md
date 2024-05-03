# Filtering and ordering by nested fields

## Purpose

Some databases, e.g. MongoDB and other document databases, allow collections to contain "columns" with nested structures (i.e. "objects") rather than just scalar values.
It would be useful to allow queries on such collections to be filtered and sorted based on the values in nested fields rather than just top-level scalar column values.

## Proposal

Add a new type `ColumnSelector`:

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ColumnSelector {
    Path(Vec<String>),
    Column(String),
}
```
The JSON representation of this type is either a string, referring to a top-level column in a collection, or an array of strings giving a path to a nested field.
In the array representation, the first element refers to the top-level column name in the collection and the other elements (if any) describe a path to a field within a nested object. The array must not be empty.

We modify `ComparisonTarget` and `OrderByTarget` to use `ColumnSelector` as follows:

```rust
pub enum ComparisonTarget {
    Column {
        name: ColumnSelector,
        path: Vec<PathElement>,
    },
    RootCollectionColumn {
        name: ColumnSelector,
    },
}

pub enum OrderByTarget {
    Column {
        name: ColumnSelector,
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

The `name` fields, which were previously just `String`s, are now `ColumnSelectors`.
Backwards compatibility is maintained because the JSON representation of `ColumnSelector::Column` is the same as `String`.

## Future extensions

Although out of scope for this RFC, in the future we probably want to extend aggregates to allow aggregating on values of nested fields.
This could be achieved by using `ColumnSelector` in `Aggregate::ColumnCount::column` and `Aggregate::SingleColumn::column`.
We could also order by aggregates on nested fields by using `ColumnSelector` in `OrderByTarget::SingleColumnAggregate::column`.