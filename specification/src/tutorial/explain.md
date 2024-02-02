# Explain

The `/query/explain` and `/mutation/explain` endpoints are not implemented in the reference implementation, simply because the `QueryResponse` is interpreted directly in the `/query` endpoint.
There is no intermediate representation (such as SQL) which could be described as a "query plan".

The `query.explain` and `mutation.explain` capabilities are turned off in the [capabilities endpoint](./capabilities.md),
and the `/query/explain` and `/mutation/explain` endpoints throw an error:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:query_explain}}
```
