# Aggregates

Now that we have computed the sorted, filtered, and paginated rows of the original collection, we can compute any aggregates over those rows.

Each aggregate is computed in turn by the `eval_aggregate` function, and added to the list of all aggregates to return:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:execute_query_aggregates}}
```

The `eval_aggregate` function works by pattern matching on the type of the aggregate being computed:

- A `star_count` aggregate simply counts all rows,
- A `column_count` aggregate computes the subset of rows where the named column is non-null, and returns the count of only those rows,
- A `single_column` aggregate is computed by delegating to the `eval_aggregate_function` function, which computes a custom aggregate operator over the values of the selected column taken from all rows.

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_aggregate}}
```

The `eval_aggregate_function` function implements the custom aggregate operators `min` and `max`, which are provided for integer-valued columns:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_aggregate_function}}
```