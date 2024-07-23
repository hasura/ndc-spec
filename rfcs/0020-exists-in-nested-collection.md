# Exists in Nested Collections

## Purpose

[Nested collections](./0019-nested-collections.md) now exist for selection as subfields, as an alternative to relationships. However, we can also extend _exists predicates_ in the same way - with support for both related collections and nested collections.

## Proposal

Add the following to `ExistsInCollection`:

```rust
pub enum ExistsInCollection {
    ...
    NestedCollection {
        column_name: FieldName,
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        arguments: BTreeMap<ArgumentName, Argument>,
        /// Path to a nested collection via object columns
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        field_path: Vec<FieldName>,
    }
}
```

`ExistsInCollection::NestedCollection` picks out a nested collection (in the same sense as [the original RFC](./0019-nested-collections.md)) as the target of an exists predicate.

As an example, we can query `institutions` by the existence of a staff member in the reference implementation - [example request](../ndc-reference/tests/query/predicate_with_exists_in_nested_collection/request.json).