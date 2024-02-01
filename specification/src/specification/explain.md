# Explain

There are two endpoints related to explain:

- The `/explain` endpoint, which accepts a [query](./queries/README.md) request.
- The `/explain/mutation` endpoint, which accepts a [mutation](./mutation/README.md) request.

Both endpoints return a representation of the _execution plan_ without actually executing the query or mutation.

## Request

```
POST /explain
```

See [`QueryRequest`](../reference/types.md#queryrequest)

## Request

```
POST /explain/mutation
```

See [`MutationRequest`](../reference/types.md#mutationrequest)

## Response

See [`ExplainResponse`](../reference/types.md#explainresponse)
