# Nested Field Selection

## Purpose

Field selection is useful in a variety of contexts:

- Document databases contain nested structures, and more often than not, we only want to select a subset of the available data.
- Many relational databases also have semi-structured datatypes like `json(b)`.
- The TypeScript connector can return object and array types, and we can pass along the selection set and only return the requested data.

This proposal is a non-breaking, minimal addition to the specification, which adds _selection_ for nested structures. Additional operations on this sort of data (filtering, sorting, aggregation, etc.) will be covered in later proposals.

## Proposal 

We extend `Field::Column` with an optional `NestedField` property which allows for selection within nested arrays and fields of nested objects:

```rust
pub enum Field {
    Column {
        column: String,
        nested_field: Option<NestedField>
    },
    Relationship {
        query: Box<Query>,
        /// The name of the relationship to follow for the subquery
        relationship: String,
        /// Values to be provided to any collection arguments
        arguments: BTreeMap<String, RelationshipArgument>,
    },
}
```

If `nested_field == None` then the column is returned in its entirety (ensuring backwards-compatibility).

A `NestedField` can be either an object or an array:

```rust
pub enum NestedField {
    Object(NestedObject),
    Array(NestedArray)
}

pub struct NestedObject {
    pub fields: IndexMap<String, Field>,
}

pub struct NestedArray {
    pub field: Box<NestedField>,
}
```

A `NestedField` contains the field selections for the currently focused substructure. 

A `NestedObject` contains a collection of `Field`s in the same sense as a top-level `Query`, but it should be evaluated in the context of the currently focused substructure, which ought to be a value with an object type.

> [!NOTE]  
> This means that a `NestedObject` can contain relationships.
> 
> How does a relationship get evaluated in the context of a nested object? Are column mappings evaluated in the context of the focused object, or the current top-level row? What about relationship arguments?

A `NestedArray` contains another `NestedField`, which indicates the field selections for each of an array of elements in turn. The intended semantics is to _map_ the selection function denoted by the contained `NestedField` over the focused substructure, which ought to be a value with an array type. In future we may want to provide other functions on nested arrays.