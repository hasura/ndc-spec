# Queries

The reference implementation of the `/query` endpoint may seem complicated, because there is a lot of functionality packed into a single endpoint. However, we will break the implementation down into small sections, each of which should be easily understood.

We start by looking at the type signature of the `post_query` function, which is the top-level function implementing the query endpoint:

```rust,no_run,noplayground
{{#include ../../../../ndc-reference/bin/reference/main.rs:post_query_signature}}
```

This function accepts a [`QueryRequest`](../../reference/types.md#queryrequest) and must produce a [`QueryResponse`](../../reference/types.md#queryresponse).

In the next section, we will start to break down this problem step-by-step.