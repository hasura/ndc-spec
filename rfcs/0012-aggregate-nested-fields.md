# Aggregates for Nested Fields

## Purpose

We can filter and sort by nested fields, but we cannot aggregate their values. This proposal addresses that missing feature.

## Proposal

- Add a new sub-capability `query.nested_fields.aggregates`.
- Add a new field `field_path: Option<Vec<String>>` to `Aggregate::ColumnCount` and `Aggregate::SingleColumn`.
  - This field may only be non-null and non-empty if the capability is turned on.
  - The field path has the same meaning as for filtering and sorting: it is a chain of object properties to traverse to reach the value to aggregate.