# Grouping

In addition to [field selection](./field-selection.md) and [computing aggregates](./aggregates.md), we also need to return the results of any requested [grouping operations](../../../specification/queries/grouping.md).

This is done by delegating to the `eval_groups` function:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:execute_query_groups}}
```

`eval_groups` takes a set of rows, and proceeds largely like `execute_query` itself.

First, rows are partitioned into groups:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_groups_partition}}
```

The `eval_dimensions` function computes a vector of dimensions for each row:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_dimensions}}
```

The only type of dimension we need to handle is a column. First the value of the column is computed by delegating to `eval_column_field_path`, and then any [extraction function](../../../specification/schema/scalar-types.md#extraction-functions) is evaluated using the `eval_extraction` function:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_dimension}}
```

Next, the partitions are sorted, using the `group_sort` function which is very similar to its row-based counterpart `sort`:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_groups_sort}}
```

Next, groups are aggregated and filtered:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_groups_filter}}
```

The `eval_group_expression` function is also very similar to the `eval_expression` function which performs a similar operation on rows.

Finally, the groups are paginated and returned:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_groups_paginate}}
```
