# Explain

The `/query/explain` and `/mutation/explain` endpoints are not implemented in the reference implementation, because their respective request objects are interpreted directly. There is no intermediate representation (such as SQL) which could be described as an "execution plan".

The `query.explain` and `mutation.explain` capabilities are turned off in the [capabilities endpoint](./capabilities.md),
and the `/query/explain` and `/mutation/explain` endpoints throw an error:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:query_explain}}
```
