# Executing Queries

In this section, we will break down the implementation of the `execute_query` function:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:execute_query_signature}}
```

At this point, we have already computed the full collection, which is passed via the `collection` argument. Now, we need to evaluate the [`Query`](../../../reference/types.md#query) in the context of this collection.

The `Query` describes the predicate which should be applied to all rows, the sort order, pagination options, along with any aggregates to compute and fields to return.

The first step is to sort the collection. 

_Note_: we could also start by filtering, and then sort the filtered rows. Which is more efficient depends on the data and the query, and choosing between these approaches would be the job of a _query planner_ in a real database engine. However, this is out of scope here, so we make an arbitrary choice, and sort the data first.