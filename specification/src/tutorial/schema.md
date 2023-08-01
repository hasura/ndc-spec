# Schema

The [schema endpoint](../specification/schema/README.md) should return data describing the data connector's scalar and object types, along with any collections, functions and procedures which are exposed.

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema1}}
    // ...
{{#include ../../../ndc-reference/bin/reference/main.rs:schema2}}
```

## Scalar Types

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_scalar_types}}
```

## Object Types

For each collection, we define an object type for its rows:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_object_types}}
```

### Author

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_object_type_author}}
```

### Article

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_object_type_article}}
```

## Collections

We define each collection's schema using the type information defined above:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_collections}}
```

### Author

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_collection_author}}
```

### Article

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_collection_article}}
```

## Functions

The schema defines a list of functions, each including its input and output [types](../specification/types.md).

### Get Latest Article

_TODO_

## Procedures

The schema defines a list of procedures, each including its input and output [types](../specification/types.md).

### Upsert Article

As an example, we define an _upsert_ procedure for the article collection defined above. The procedure will accept an input argument of type `article`, and returns a nulcollection `article`, representing the state of the article before the update, if it were already present.

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_procedure_upsert_article}}
```