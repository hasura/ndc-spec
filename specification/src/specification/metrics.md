# Metrics

Data connectors should provide a __metrics endpoint__ which reports relevant metrics in a textual format. Data connectors can report any metrics which are deemed relevant, or none at all, with the exception of any reserved keys.

## Request

```
GET /metrics
```

## Response

The metrics endpoint should return a content type of `text/plain`, and return any metrics in the [Prometheus textual format](https://prometheus.io/docs/instrumenting/exposition_formats/#text-based-format).

### Reserved keys

Metric names prefixed with `hasura_` are reserved for future use, and should not be included in the response.

## Example

```
# HELP query_total The number of /query requests served
# TYPE query_total counter
query_total 10000 1685405427000
# HELP mutation_total The number of /mutation requests served
# TYPE mutation_total counter
mutation_total 5000 1685405427000
```