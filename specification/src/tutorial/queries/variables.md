# Query Variables

The first step in `post_query` is to reduce the problem from a query with multiple sets of [query variables](../../specification/queries/variables.md) to only a single set.

The `post_query` function iterates over all variable sets, and for each one, produces a [`RowSet`](../../reference/types.md#rowset) of rows corresponding to that set of variables. Each `RowSet` is then added to the final `QueryResponse`:

```rust,no_run,noplayground
{{#include ../../../../ndc-reference/bin/reference/main.rs:post_query}}
```

In order to compute the `RowSet` for a given set of variables, the function delegates to a function named `execute_query_with_variables`:

```rust,no_run,noplayground
{{#include ../../../../ndc-reference/bin/reference/main.rs:execute_query_with_variables_signature}}
```

In the next section, we will break down the implementation of this function.