# Field Selection

A [`Query`](../../reference/types.md#query) can specify which fields to fetch. The available fields are either

- the columns on the selected table (i.e. those advertised in the corresponding [`TableInfo`](../../reference/types.md#tableinfo) structure in the [schema response](../schema/tables.md)), or
- fields from [related tables](./relationships.md)

The requested fields are specified as a collection of [`Field`](../../reference/types.md#field) structures in the `field` property on the [`Query`](../../reference/types.md#query).

## Example

Here is an example of a query which selects some columns from the `articles` table of the reference data connector:

```json
{{#include ../../../../ndc-reference/tests/query/get_all_articles/request.json}}
```

## Requirements

- If the [`QueryRequest`](../../reference/types.md#queryrequest) contains a [`Query`](../../reference/types.md#query) which specifies `fields`, then each [`RowSet`](../../reference/types.md#rowset) in the response should contain the `rows` property, and each row should contain all of the requested fields.

## See also

- Type [`Query`](../../reference/types.md#query)
- Type [`RowFieldValue`](../../reference/types.md#rowfieldvalue)
- Type [`RowSet`](../../reference/types.md#rowset)
