# Explain

There are two endpoints related to explain:

- The `/query/explain` endpoint, which accepts a [query](./queries/README.md) request.
- The `/mutation/explain` endpoint, which accepts a [mutation](./mutation/README.md) request.

Both endpoints return a representation of the _execution plan_ without actually executing the query or mutation.

Connectors that wish to support these endpoints should indicate this in their capabilities; specifically with the `query.explain` capability and the `mutation.explain` capability.

## Request

```
POST /query/explain
```

See [`QueryRequest`](../reference/types.md#queryrequest)

## Request

```
POST /mutation/explain
```

See [`MutationRequest`](../reference/types.md#mutationrequest)

## Response

See [`ExplainResponse`](../reference/types.md#explainresponse)
