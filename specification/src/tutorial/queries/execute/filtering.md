# Filtering

The next step is to filter the rows based on the provided predicate expression:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:execute_query_filter}}
```

As we can see, the function delegates to the `eval_expression` function in order to evaluate the predicate on each row.

## Evaluating expressions

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_signature}}
```