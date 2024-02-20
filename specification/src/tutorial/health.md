# Health and Metrics

## Service Health

The `/health` endpoint has nothing to check, because the reference implementation does not need to connect to any other services. Therefore, once the reference implementation is running, it can always report a healthy status:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:health}}
```

In practice, a connector should make sure that any upstream services can be successfully contacted, and respond accordingly.

## Metrics

The reference implementation maintains some generic access metrics in its application state:

- `metrics.total_requests` counts the number of requests ever served, and
- `metrics.active_requests` counts the number of requests _currently_ being served.

The [metrics endpoint](../specification/metrics.md) reports these metrics using the Rust [prometheus](https://docs.rs/prometheus/latest/prometheus/) crate:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:metrics}}
```

To maintain these metrics, it uses a simple metrics middleware:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:metrics_middleware}}
```