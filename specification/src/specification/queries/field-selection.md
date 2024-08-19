# Field Selection

A [`Query`](../../reference/types.md#query) can specify which fields to fetch. The available fields are either

- the columns on the selected collection (i.e. those advertised in the corresponding [`CollectionInfo`](../../reference/types.md#collectioninfo) structure in the [schema response](../schema/collections.md)), or
- fields from [related collections](./relationships.md)

The requested fields are specified as a collection of [`Field`](../../reference/types.md#field) structures in the `field` property on the [`Query`](../../reference/types.md#query).

## Field Arguments

Arguments can be supplied to fields via the `arguments` key. These match the format described in [the arguments documentation](../arguments.md).

The [schema response](../schema/object-types.md) will specify which fields take arguments via its respective `arguments` key.

If a field has any arguments defined, then the `arguments` field must be provided wherever that field is referenced. All fields are required, including nullable fields.

## Nested Fields

Queries can specify nested field selections for columns which have structured types (that is, not simply a scalar type or a nullable scalar type).

In order to specify nested field selections, the `fields` property of the `Field` structure, which is a [`NestedField`](../../reference/types.md#nestedfield) structure.

If `fields` is omitted, the entire structure of the column's data should be returned.  

If `fields` is provided, its value should be compatible with the type of the column:

### Nested objects

For an object-typed column (whether nullable or not), the `fields` property should contain a `NestedField` with type `object`. 

The `fields` property of the `NestedField` specifies a [`Field`](../../reference/types.md#field) structure for each requested nested field from the objects.

### Nested arrays

For an array-typed column (whether nullable or not), the `fields` property may contain a `NestedField` with type `array`. 

The `fields` property of the `NestedField` should contain _another_ `NestedField` structure, compatible with the type of the elements of the array. The selection function denoted by this nested `NestedField` structure should be applied to each element of each array.

### Nested collections

For a column whose type is an array of objects (whether nullable or not), the `fields` property may contain a `NestedField` with type `collection`.

A connector should handle such fields by treating the nested array of objects as a collection. Such a field will include a nested `Query`, and the connector should execute that query in the context of this nested collection.

A response for a field with a `fields` property of type `collection` should be a `RowSet` which is computed from the nested collection by executing the specified query.

_Note_: support for nested collection queries is indicated by the `query.nested_fields.nested_collections` capability.

### Nested fields and relationships

Within the scope of a nested object, that object should be used as the "current row" wherever that concept is appropriate:

- In a `Field::Column` field, the column name points to a field of the nested object,
- In a `Field::Relationship` field:
  - A [column mapping](./relationships.md#column-mappings) refers to fields from the nested object,
  - A [relationship argument](./arguments.md#relationships) which selects a column refers to fields of the nested object.
  
## Examples

### Simple column selection

Here is an example of a query which selects some columns from the `articles` collection of the reference data connector:

```json
{{#include ../../../../ndc-reference/tests/query/get_all_articles/request.json}}
```

### Example with Nested Object Types

Here is an example of a query which selects some columns from a nested object inside the rows of the `institutions` collection of the reference data connector:

```json
{{#include ../../../../ndc-reference/tests/query/nested_object_select/request.json:1}}
{{#include ../../../../ndc-reference/tests/query/nested_object_select/request.json:3:}}
```

Notice that the `location` column is fetched twice: once to illustrate the use of the `fields` property, to fetch a subset of data, and again in the `location_all` field, which omits the `fields` property and fetches the entire structure.

### Example with Nested Array Types

Here is an example of a query which selects some columns from a nested array inside the rows of the `institutions` collection of the reference data connector:

```json
{{#include ../../../../ndc-reference/tests/query/nested_array_select/request.json:1}}
{{#include ../../../../ndc-reference/tests/query/nested_array_select/request.json:3:}}
```

Notice that the `staff` column is fetched using a `fields` property of type `array`. For each staff member in each institution row, we apply the selection function denoted by its `fields` property (of type `object`). Specifically, the `last_name` and `specialities` properties are selected for each staff member.

### Example with a Nested Collection

Here is an example of a query which computes aggregates over a nested collection inside the `staff` field of each row of the `institutions` collection:

```json
{{#include ../../../../ndc-reference/tests/query/nested_collection_with_aggregates/request.json:1}}
{{#include ../../../../ndc-reference/tests/query/nested_collection_with_aggregates/request.json:3:}}
```

Note the `staff_aggregates` field in particular, which has `fields` with type `collection`.

### Example with Nested Types and Relationships

This query selects `institution` data, and fetches `author` data if the first and last name fields match for any nested `staff` objects:


```json
{{#include ../../../../ndc-reference/tests/query/nested_object_select_with_relationship/request.json:1}}
{{#include ../../../../ndc-reference/tests/query/nested_object_select_with_relationship/request.json:3:}}
```

Note that the `first_name` and `last_name` properties in the column mapping are evaluated in the context of the nested `staff` object, and _not_ in the context of the original `institution` row.

### Example with Field Arguments

Here is an example of a query which selects some columns from a nested array inside the rows of the `institutions` collection of the reference data connector and uses the `limit` field argument to limit the number of items returned:

```json
{{#include ../../../../ndc-reference/tests/query/nested_array_select_with_limit/request.json:1}}
{{#include ../../../../ndc-reference/tests/query/nested_array_select_with_limit/request.json:3:}}
```

## Requirements

- If the [`QueryRequest`](../../reference/types.md#queryrequest) contains a [`Query`](../../reference/types.md#query) which specifies `fields`, then each [`RowSet`](../../reference/types.md#rowset) in the response should contain the `rows` property, and each row should contain all of the requested fields.

## See also

- Type [`Query`](../../reference/types.md#query)
- Type [`RowFieldValue`](../../reference/types.md#rowfieldvalue)
- Type [`RowSet`](../../reference/types.md#rowset)
