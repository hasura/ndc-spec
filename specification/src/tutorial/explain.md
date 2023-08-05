# Explain

The `/explain` endpoint is not implemented in the reference implementation, simply because tje `QueryResponse` is interpreted directly in the `/query` endpoint. There is no intermediate representation (such as SQL) which could be described as a "query plan".

The `explain` capability is turned off in the [capabilities endpoint](./capabilities.md), and the `/explain` endpoint throws an error:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:explain}}
```
