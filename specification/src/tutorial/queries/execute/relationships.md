# Relationships

Relationships appear in many places in the [`QueryRequest`](../../../reference/types.md#queryrequest), but are always computed using the `eval_path` function.

`eval_path` accepts a list of [`PathElement`](../../../reference/types.md#pathelement)s, each of which describes the traversal of a single edge of the collection-relationship graph. `eval_path` computes the collection at the final node of this path through the graph.

It does this by successively evaluating each edge in turn using the `eval_path_element` function:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_path}}
```

The `eval_path_element` function computes a collection from a single relationship, one source row at a time, by evaluating all relationship arguments, computing the target collection using `get_collection_by_name`, and evaluating any column mapping on any resulting rows:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_path_element}}
```