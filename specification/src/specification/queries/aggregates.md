# Aggregates

In addition to fetching multiple rows of raw data from a collection, the query API supports fetching aggregated data.

Aggregates are requested in the `aggregates` field of the [`Query`](../../reference/types.md#query) object.

There are three types of aggregate:

- `single_column` aggregates apply an aggregation function (as defined by the column's [scalar type](../schema/scalar-types.md) in the schema response) to a column,
- `column_count` aggregates count the number of rows with non-null values in the specified columns. If the `distinct` flag is set, then the count should only count unique non-null values of those columns,
- `star_count` aggregates count all matched rows.

If the connector supports capability `query.nested_fields.aggregates` then `single_column` and `column_count` aggregates may also [reference nested fields within a column](./filtering.md#referencing-nested-fields-within-columns) using the `field_path` property.

## Example

The following query object requests the aggregated sum of all order totals, along with the count of all orders, and the count of all orders which have associated invoices (via the nullable `invoice_id` column):

```json
{
  "collection": ["orders"],
  "collection_relationships": {},
  "query": {
    "aggregates": {
      "orders_total": {
        "type": "single_column",
        "function": "sum",
        "column": "total"
      },
      "invoiced_orders_count": {
        "type": "column_count",
        "columns": ["invoice_id"]
      },
      "orders_count": {
        "type": "star_count"
      }
    }
  }
}
```

In this case, the query has no predicate function, so all three aggregates would be computed over all rows.

## Requirements

- Each aggregate should be computed over all rows that match the `Query`.
- Each requested aggregate must be returned in the `aggregates` property on the [`QueryResponse`](../../reference/types.md#queryresponse) object, using the same key as used to request it.

## See also

- Type [`Aggregate`](../../reference/types.md#aggregate)