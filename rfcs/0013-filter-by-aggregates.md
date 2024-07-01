# Filter by Aggregates

## Purpose

We can order by aggregates, but not filter. This proposal adds filtering capabilities based on the values of aggregates on related tables.

## Proposal

- Add a new sub-capability `capabilities.relationships.filter_by_aggregate`.
- Add a new alternative to `ComparisonTarget`:

  ```rust
  pub enum ComparisonTarget {
      ...
      Aggregate {
          /// Aggregation method to use
          aggregate: Aggregate,
          /// Non-empty collection of relationships to traverse
          path: Vec<PathElement>,
      },
  }
  ```
- Break `OrderByTarget` while we have the chance as well, to reuse the same structure (and support column count aggregates):
  ```rust
  pub enum OrderByTarget {
      ...
      Aggregate {
          /// Aggregation method to use
          aggregate: Aggregate,
          /// Non-empty collection of relationships to traverse
          path: Vec<PathElement>,
      },
  }
  ```