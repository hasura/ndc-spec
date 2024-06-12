# Group By

## Purpose

Just as we can aggregate row sets, it is also useful to be able to group row sets into chunks, which can be aggregated independently, generalizing SQL's `GROUP BY` functionality.

While we could consider adding this functionality on a per-connector basis using functions, it is desirable to reuse existing functionality: collections can be filtered and sorted to form a rowset before aggregation (meaning that permissions in v3-engine can be easily applied to group-by queries), and aggregation operators and return types can be reused from the aggregations feature.

## Proposal

Alongside `fields` and `aggregates`, we'll add a new field `groups` to `Query`:

```rust
pub struct Query {
    pub aggregates: Option<IndexMap<String, Aggregate>>,
    pub fields: Option<IndexMap<String, Field>>,
    // ...
    pub groups: Option<Grouping>,
}
```

The intent is that the same row set should be grouped and aggregated if `groups` is specified.

The `Grouping` structure looks like this:

```rust
pub struct Grouping {
    /// Dimensions along which to partition the data
    pub dimensions: IndexMap<String, Dimension>,
    /// Aggregates to compute in each group
    pub aggregates: IndexMap<String, Aggregate>,
}
```

A `Grouping` slices a row set along several dimensions, and then performs some aggregations over each slice.

A `Dimension` is an enum, but there is only one option right now, which is to slice by the value of a column:

```rust
pub enum Dimension {
    Column {
        /// The name of the column
        column_name: String,
        /// Path to a nested field within an object column
        field_path: Option<Vec<String>>,
    },
}
```

We will support nested fields as dimensions.

We might want to extend this enum later, for example with related collections, or multi-faceted dimensions (like timestamps faceted by various resolutions), or other kinds of dimensions.

`Aggregate` is just the usual type of aggregates.

The `RowSet` type is also extended to include `groups` when requested:

```rust
pub struct RowSet {
    /// The results of the aggregates returned by the query
    pub aggregates: Option<IndexMap<String, serde_json::Value>>,
    /// The rows returned by the query, corresponding to the query's fields
    pub rows: Option<Vec<IndexMap<String, RowFieldValue>>>,
    /// The results of any grouping operation applies to the returned rows
    pub groups: Option<Vec<Group>>,
}
```

A `Group` is defined as follows:

```rust
pub struct Group {
    /// Values of dimensions which identify this group
    pub dimensions: IndexMap<String, serde_json::Value>,
    /// Aggregates computed within this group
    pub aggregates: IndexMap<String, serde_json::Value>,
}
```

Requirements:

- If `groups` is specified, then a `groups` property should also be provided in each row set returned.
- The returned groups should correspond to a partition of the row set computed from the query:
  - Within each partition, the chosen dimensions must be equal. These common values are hoisted out to the `dimensions` property of the `Group`.
  - `aggregates` are computed over each partition in turn, in the same way as the aggregates API would compute them over the selected rows.

The exact semantics of this partitioning operation are unspecified. For example, one connector might implement sort-then-group, and another might choose not to sort and return multiple groups for the same dimensions instead.

## Future Work

- Support ordering and filtering after the grouping operation
  - Post-group filtering is called `HAVING` in SQL.

## Open Questions

- A `Grouping` with no dimensions is just a regular aggregation, do we want to unify the two?
