# Capabilities

The [capabilities endpoint](../specification/capabilities.md) should return data describing which features the data connector can implement, along with the version of this specification that the data connector claims to implement.

The reference implementation returns a static `CapabilitiesResponse`:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:capabilities}}
```

_Note_: the reference implementation supports all capabilities with the exception of `query.explain` and `mutation.explain`. This is because all queries are run in memory by naively interpreting the query request - there is no better description of the query plan than the raw query request itself!
