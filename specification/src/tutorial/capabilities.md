# Capabilities

The [capabilities endpoint](../specification/capabilities.md) should return data describing which features the data connector can implement, along with a range of versions of this specification that the data connector claims to implement.

The reference implementation returns a static `CapabilitiesResponse`:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:capabilities}}
```