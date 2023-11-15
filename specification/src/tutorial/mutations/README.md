# Mutations

In this section, we will break down the implementation of the `/mutation` endpoint.

The mutation endpoint is handled by the `post_mutation` function:

```rust,no_run,noplayground
{{#include ../../../../ndc-reference/bin/reference/main.rs:post_mutation_signature}}
```

This function receives the application state, and the [`MutationRequest`](../../reference/types.md#mutationrequest) structure.

The function iterates over the collection of requested [`MutationOperation`](../../reference/types.md#mutationoperation) structures, and handles each one in turn, adding each result to the `operation_results` field in the response:

```rust,no_run,noplayground
{{#include ../../../../ndc-reference/bin/reference/main.rs:post_mutation}}
```

The `execute_mutation_operation` function is responsible for executing an individual operation. In the next section, we'll break that function down.