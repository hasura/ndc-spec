# Sorting

A [`Query`](../../reference/types.md#query) can specify how rows should be sorted in the response.

The requested ordering can be found in the `order_by` field of the [`Query`](../../reference/types.md#query) object.

## Computing the Ordering

To compute the ordering from the `order_by` field, data connectors should implement the following ordering between rows:

- Consider each element of the `order_by.elements` array in turn.
- For each [`OrderByElement`](../../reference/types.md#orderbyelement):
  - If `element.target.type` is `column`, then to compare two rows, compare the value in the selected column. See type `column` below.
  - If `element.target.type` is `star_count_aggregate`, compare two rows by comparing the row count of a related collection. See type `star_count_aggregate` below.
  - If `element.target.type` is `single_column_aggregate`, compare two rows by comparing a single column aggregate. See type `single_column_aggregate` below.

### Type `column`

The property `element.target.name` refers to a column name.
If the connector supports capability `query.nested_fields.order_by` then the target may also [reference nested fields within a column](./filtering.md#referencing-nested-fields-within-columns) using the `field_path` property.

If `element.order_direction` is `asc`, then the row with the smaller column comes first. 

If `element.order_direction` is `asc`, then the row with the smaller column comes second. 

If the column values are incomparable, continue to the next [`OrderByElement`](../../reference/types.md#orderbyelement).

The data connector should document, for each scalar type, a comparison function to use for any two values of that scalar type.

For example, a data connector might choose to use the obvious ordering for a scalar integer-valued type, but to use the database-given ordering for a string-valued type, based on a certain choice of collation.

For example, the following `query` requests that a collection of articles be ordered by `title` descending:

```json
{{#include ../../../../ndc-reference/tests/query/order_by_column/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/order_by_column/request.json:3: }}
```

The selected column can be chosen from a related collection by specifying the `path` property. `path` consists of a list of named [relationships](./relationships.md).

For example, this query sorts articles by their author's last names, and then by their first names, by traversing the relationship from articles to authors:

```json
{{#include ../../../../ndc-reference/tests/query/order_by_relationship/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/order_by_relationship/request.json:3: }}
```

### Type `star_count_aggregate`

An ordering of type `star_count_aggregate` orders rows by a count of rows in some [related collection](./relationships.md). If the respective counts are incomparable, the ordering should continue to the next [`OrderByElement`](../../reference/types.md#orderbyelement).

For example, this query sorts article authors by their total article count:

```json
{{#include ../../../../ndc-reference/tests/query/order_by_aggregate/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/order_by_aggregate/request.json:3: }}
```

### Type `single_column_aggregate`

An ordering of type `single_column_aggregate` orders rows by an aggregate computed over rows in some [related collection](./relationships.md). If the respective aggregates are incomparable, the ordering should continue to the next [`OrderByElement`](../../reference/types.md#orderbyelement).

For example, this query sorts article authors by their maximum article ID:

```json
{{#include ../../../../ndc-reference/tests/query/order_by_aggregate_function/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/order_by_aggregate_function/request.json:3: }}
```

## Requirements

- Rows in the response should be ordered according to the algorithm described above.
- The `order_by` field should not affect the set of collection which are returned, except for their order.
- If the `order_by` field is not provided then rows should be returned in an unspecified but deterministic order. For example, an implementation might choose to return rows in the order of their primary key or creation timestamp by default.

## See also

- Type [`OrderBy`](../../reference/types.md#orderby)
- Type [`OrderByElement`](../../reference/types.md#orderbyelement)
- Type [`OrderByTarget`](../../reference/types.md#orderbytarget)