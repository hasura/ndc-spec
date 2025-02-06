# Pagination

The `limit` and `offset` parameters on the [`Query`](../../reference/types.md#query) object control pagination:

- `limit` specifies the maximum number of rows that are considered during field selection and before aggregates and grouping are applied.
- `offset`: The index of the first row to consider during field selection and before aggregates and grouping are applied.

`limit` and `offset` are applied after the [predicate filter from the Query](../filtering.md) is applied and after [sorting from the Query](../sorting.md) is applied, but before [aggregates](../aggregates.md) and [grouping](../grouping.md) are applied. Both `limit` and `offset` affect the rows returned by field selection.

## Requirements

- If `limit` is specified, the response should contain at most that many rows, and aggregates and grouping should be applied to at most that many rows.

## See also

- Type [`Query`](../../reference/types.md#query)
