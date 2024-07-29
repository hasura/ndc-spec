# Grouping

If a connector supports [aggregates](./aggregates.md), it may also support _grouping_ data and then aggregating data in those groups. This ability is tracked by the `query.aggregates.group_by` capability.

Grouping is requested in the query API alongside fields and aggregates, in the `groups` field of the [`Query`](../../reference/types.md#query) object.

A grouping operation specifies one or more _dimensions_ along which to partition the row set. For each group, every row should have equal values in each of those dimension columns.

In addition, a grouping operation specifies _aggregates_ which should be computed and returned for each group separately.

## Filtering

Grouping operations have two types of filtering:

- The initial row set can be filtered _before the grouping operation_, using the `predicate` field of the [`Query`](../../reference/types.md#query) object as usual, and
- The _groups themselves_ can be filtered _after the grouping operation_, using the `predicate` field of the [`Grouping`](../../reference/types.md#grouping) object. This is controlled by the `queries.aggregates.group_by.filter` capability.

Unlike regular predicates on rows, group predicates are not allowed to compare _columns_, but must instead compare values of _aggregates_ over the group. For example, we can filter groups by comparing a _count_ of rows in the group, but not by comparing values in individual rows.

## Ordering

As with filtering, group operations support two types of ordering:

- The initial row set can be ordered _before the grouping operation_, using the `order_by` field of the [`Query`](../../reference/types.md#query) object as usual, and
- The _groups themselves_ can be ordered _after the grouping operation_, using the `order_by` field of the [`Grouping`](../../reference/types.md#grouping) object. This is controlled by the `queries.aggregates.group_by.order` capability.

And just as with filtering, group sort orders are restricted to comparing aggregate values. For example, we can order groups by a _count_, but not by the value of individual rows.

## Pagination

Pagination can also be applied both before and after grouping:

- The initial row set can be paginated _before the grouping operation_, using the `limit` and `offset` fields of the [`Query`](../../reference/types.md#query) object as usual, and
- The _groups themselves_ can be paginated _after the grouping operation_, using the `limit` and `offset` fields of the [`Grouping`](../../reference/types.md#grouping) object. This is controlled by the `queries.aggregates.group_by.paginate` capability.

## Examples

This example partitions the `articles` collection by `author_id`, and then returns the row count for each group. That is, it computes the number of articles written by each author:

```json
{{#include ../../../../ndc-reference/tests/query/group_by_with_star_count/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/group_by_with_star_count/request.json:3: }}
```

### Filtering examples

This example applies a predicate to the rows _before grouping_:

```json
{{#include ../../../../ndc-reference/tests/query/group_by_with_where/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/group_by_with_where/request.json:3: }}
```

This example applies a predicate to the groups themselves, _after grouping_. It computes some aggregates for author groups which have exactly two articles:

```json
{{#include ../../../../ndc-reference/tests/query/group_by_with_having/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/group_by_with_having/request.json:3: }}
```

### Ordering and pagination

This example computes the article count for the author with the most articles, by ordering the groups by article count, and then using pagination to select the first group:

```json
{{#include ../../../../ndc-reference/tests/query/group_by_with_order_by/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/group_by_with_order_by/request.json:3: }}
```

## Requirements

- If the [`Query`](../../reference/types.md#query) object specifies the `groups` field, then each correponding [`RowSet`](../../reference/types.md#rowset) object must contain a non-null `groups` field.
- Each returned [`Group`](../../reference/types.md#group) object must contain values for each requested dimension, in the order in which they were requested:
  - The connector should effectively partition the [`RowSet`](../../reference/types.md#rowset) described by the [`Query`](../../reference/types.md#query) object into groups, such that the dimension tuples are unique within each group.
- Each returned [`Group`](../../reference/types.md#group) object must contain values for each requested aggregate, using the same key as used to request it:
  - Aggregates should be computed over the rows in each group in turn.

## See also

- Type [`Aggregate`](../../reference/types.md#aggregate)
- Type [`Dimension`](../../reference/types.md#dimension)
- Type [`Group`](../../reference/types.md#group)
- Type [`Grouping`](../../reference/types.md#grouping)
