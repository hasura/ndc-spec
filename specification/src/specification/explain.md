# Explain

The explain endpoint accepts an explain request, which is either a [query](./queries/README.md) request or a [mutation](./mutation/README.md) request, returns a representation of the _execution plan_ without actually executing the query or mutation.

## Request

```
POST /explain
```

See [`ExplainRequest`](../reference/types.md#explainrequest)

## Response

See [`ExplainResponse`](../reference/types.md#explainresponse)
