# Field Selection

In addition to computing aggregates, we can also return fields selected directly from the rows themselves.

This is done by mapping over the computed rows, and using the `eval_field` function to evaluate each selected field in turn:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:execute_query_fields}}
```

The `eval_field` function works by pattern matching on the field type:

- A `column` is selected using the `eval_column` function (or `eval_nested_field` if there are nested fields to fetch)
- A `relationship` field is selected by evaluating the related collection using `eval_path_element` (we will cover this in the next section), and then recursively executing a query using `execute_query`:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_field}}
```