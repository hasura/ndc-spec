# Relationships

Queries can request data from other collections via relationships. A relationship identifies rows in one collection (the "source collection") with possibly-many related rows in a second collection (the "target collection") in two ways:

- Columns in the two collections can be related via _column mappings_, and
- [Collection arguments](./arguments.md) to the target collection can be computed via the row of the source collection.

## Defining Relationships

Relationships are defined (and given names) in the top-level `QueryRequest` object, and then referred to by name everywhere they are used. To define a relationship, add a [`Relationship`](../../reference/types.md#relationship) object to the `collection_relationships` property of the `QueryRequest` object.

## Column Mappings

A column mapping is a set of pairs of columns - each consisting of one column from the source object type and one column from the target collection - which must be pairwise equal in order for a pair of rows to be considered equal.

What the source object type is depends on where the relationship is used. Often, a relationship will simply relate columns from one source collection's object type to a target collection's object type. However, at various locations such as field selection, filtering, ordering, grouping, queries can descend into nested objects and arrays before navigating the relationship. In these cases, the source column will be on the nested object type. Only connectors that enable the `relationships.nested` capability will encounter relationships that involve nested objects. Additionally, only connectors that enable the `relationships.nested.array` capability will encounter relationships that start from inside nested objects in nested arrays.

The column from the target collection may be an object-nested column, so it is specified using a field path to the column. An array of one field name specifies a column on the target collection's object type. Two field names specifies, firstly, the column on the target collection that contains a nested object, and secondly the column on the nested object type.

However, unless a connector enables the `relationships.nested` capability, it can expect to only receive field paths with only one entry in column mappings (ie. non-nested columns).

For example, we can fetch each `author` with its list of related `articles` by establishing a column mapping between the author's primary key and the article's `author_id` column:

```json
{{#include ../../../../ndc-reference/tests/query/authors_with_articles/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/authors_with_articles/request.json:3: }}
```

## Collection Arguments

See [collection arguments](./arguments.md) for examples.

## Advanced relationship use cases

Relationships are not used only for fetching data - they are used in practically all features of data connectors, as we will see below.

### Relationships in predicates

`EXISTS` expressions in predicates can query related collections. Here we find all authors who have written any article with `"Functional"` in the title:

```json
{{#include ../../../../ndc-reference/tests/query/predicate_with_exists/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/predicate_with_exists/request.json:3: }}
```

### Relationships in `order_by`

Sorting can be defined in terms of row counts and aggregates over related collections.

For example, here we order authors by the number of articles they have written:

```json
{{#include ../../../../ndc-reference/tests/query/order_by_aggregate/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/order_by_aggregate/request.json:3: }}
```

We can also order by custom aggregate functions applied to related collections. For example, here we order authors by their most recent (maximum) article ID:

```json
{{#include ../../../../ndc-reference/tests/query/order_by_aggregate_function/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/order_by_aggregate_function/request.json:3: }}
```
