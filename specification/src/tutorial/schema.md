# Schema

The [schema endpoint](../specification/schema/README.md) should return data describing the data connector's scalar and object types, along with any tables and commands which are exposed.

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

For each table, we define an object type for its rows:

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

## Tables

We define each table's schema using the type information defined above:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_tables}}
```

### Author

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_table_author}}
```

### Article

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_table_article}}
```

## Commands

The schema defines a list of commands, each including its input and output [types](../specification/types.md).

## Upsert Article

As an example, we define an _upsert_ command for the article table defined above. The command will accept an input argument of type `article`, and returns a nullable `article`, representing the state of the article before the update, if it were already present.

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:schema_command_upsert_article}}
```