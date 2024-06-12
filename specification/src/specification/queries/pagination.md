# Pagination

The `limit` and `offset` parameters on the [`Query`](../../reference/types.md#query) object control pagination:

- `limit` specifies the maximum number of rows to return from a query in the rows property.
- `offset`: The index of the first row to return.

Both `limit` and `offset` affect the rows returned, and also the rows considered by aggregations.

## Requirements

- If `limit` is specified, the response should contain at most that many rows.

## See also

- Type [`Query`](../../reference/types.md#query)