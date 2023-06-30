# Telemetry

Hasura uses OpenTelemetry to coordinate the collection of traces and metrics with data connectors.

## Trace Collection

Trace collection is out of the scope of this specification currently. This may change in a future revision.

## Trace Propagation

Hasura uses the [W3C TraceContext specification](https://www.w3.org/TR/trace-context/) to implement trace propagation. Data connectors should propagate tracing headers in this format to any downstream services.