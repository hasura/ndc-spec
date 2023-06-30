# Pagination

The `limit` and `offset` parameters on the [`Query`](../../reference/types.md#query) object control pagination:

- `limit` specifies the maximum number of rows to return from a query in the rows property. `limit` does not influence the rows considered by aggregations.
- `offset`: The index of the first row to return. This affects the rows returned, and also the rows considered by aggregations.

## Pagination in Aggregation Queries

_TODO_

## Requirements

- If `limit` is specified, the response should contain at most that many rows.

## See also

- Type [`Query`](../../reference/types.md#query)