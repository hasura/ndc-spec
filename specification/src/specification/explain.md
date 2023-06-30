# Explain

The explain endpoint accepts a [query](./queries/README.md) request, but without actually executing the query, returns a representation of the _execution plan_.

## Request

```
POST /explain
```

## Request

See [`QueryRequest`](../reference/types.md#queryrequest)

## Response

See [`ExplainResponse`](../reference/types.md#explainresponse)