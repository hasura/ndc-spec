# Filter by Aggregates

## Purpose

We can order by aggregates, but not filter. This proposal adds filtering capabilities based on the values of aggregates on related tables.

## Proposal

- Add a new sub-capability `capabilities.relationships.filter_by_aggregate`.
- Add new alternatives to `ComparisonTarget` based on the existing members of `OrderByTarget`:

  ```rust
  pub enum ComparisonTarget {
      ...
      SingleColumnAggregate {
          /// The column to apply the aggregation function to
          column: String,
          /// Path to a nested field within an object column
          field_path: Option<Vec<String>>,
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
  ```