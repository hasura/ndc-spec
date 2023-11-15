# Handling Operations

The `execute_mutation_operation` function is responsible for handling a single [`MutationOperation`](../../reference/types.md#mutationoperation), and returning the corresponding [`MutationOperationResults`](../../reference/types.md#mutationoperationresults):

```rust,no_run,noplayground
{{#include ../../../../ndc-reference/bin/reference/main.rs:execute_mutation_operation}}
```

The function matches on the type of the operation, and delegates to the appropriate function. Currently, the only type of operation is `Procedure`, so the function delegates to the `execute_procedure` function. In the next section, we will break down the implementation of that function.