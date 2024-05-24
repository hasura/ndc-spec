# Schema

The schema endpoint should return data describing the data connector's scalar and object types, along with any collections, functions and procedures which are exposed.

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema1}}
    // ...
{{#include ../../../ndc-reference/bin/reference/main.rs:schema2}}
```

## Scalar Types

We define two scalar types: `String` and `Int`.

`String` supports a custom `like` comparison operator, and `Int` supports custom aggregation operators `min` and `max`.

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_scalar_types}}
```

## Object Types

For each collection, we define an object type for its rows. In addition, we define object types for any nested types which we use:

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

### Institution

_Note_: the fields with array types have field-level arguments (`array_arguments`) in order to support nested array operations.

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_object_type_institution}}
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

### `articles_by_author`

We define one additional collection, `articles_by_author`, which is provided as an example of a collection with an argument:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_collection_articles_by_author}}
```

### Institution

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_collection_institution}}
```

## Functions

The schema defines a list of [functions](../specification/schema/functions.md), each including its input and output [types](../specification/types.md).

### Get Latest Article

As an example, we define a `latest_article_id` function, which returns a single integer representing the maximum article ID.

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_function_latest_article_id}}
```

A second example returns the full corresponding article, to illustrate functions returning structured types:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_function_latest_article}}
```

## Procedures

The schema defines a list of [procedures](../specification/schema/procedures.md), each including its input and output [types](../specification/types.md).

### Upsert Article

As an example, we define an _upsert_ procedure for the article collection defined above. The procedure will accept an input argument of type `article`, and returns a nulcollection `article`, representing the state of the article before the update, if it were already present.

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_procedure_upsert_article}}
```