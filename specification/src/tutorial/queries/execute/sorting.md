# Sorting

The first step is to sort the rows in the full collection:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:execute_query_sort}}
```

The [`Query`](../../../reference/types.md#query) object defines the sort order in terms of a list of [`OrderByElement`](../../../reference/types.md#orderbyelement)s. See the [sorting specification](../../../specification/queries/sorting.md) for details on how this ought to be interpreted.

## The `sort` function

The `sort` function implements a simple insertion sort, computing the ordering for each pair of rows, and inserting each row at the correct place:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:sort}}
```

`sort` delegates to the `eval_order_by` function to compute the ordering between two rows:

## Evaluating the Ordering

To compare two rows, the `eval_order_by` computes each `OrderByElement` in turn, and compares the rows in order, or in reverse order, depending on whether the ordering is _ascending_ or _descending_. 

The function returns the first `Ordering` which makes the two rows distinct (if any):

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_order_by}}
```

The ordering for a single `OrderByElement` is computed by the `eval_order_by_element` function. 

We won't cover every branch of this function in detail here, but it works by pattern matching on the type of ordering being used. 

### Ordering by a column

As an example, here is the function `eval_order_by_column` which evaluates _ordering by a column_:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_order_by_column}}
```

This code computes the target table, possibly by traversing relationships using `eval_path` (we will cover this function later when we cover relationships), and validates that we computed a single row before selecting the value of the chosen column.

Now that we have sorted the full collection, we can apply the predicate to filter down the collection of rows. We will cover this in the next section.