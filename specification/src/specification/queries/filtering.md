# Filtering

A [`Query`](../../reference/types.md#query) can specify a predicate expression which should be used to filter rows in the response.

A predicate expression can be one of

- An application of a _comparison operator_ to a column and a value, or
- An `EXISTS` expression, or
- A _conjunction_ of other expressions, or
- A _disjunction_ of other expressions, or
- A _negation_ of another expression

The predicate expression is specified in the `predicate` field of the [`Query`](../../reference/types.md#query) object.

## Comparison Operators

### Unary Operators

Unary comparison operators are denoted by expressions with a `type` field of `unary_comparison_operator`.

The only supported unary operator currently is `is_null`, which return `true` when a column value is `null`:

```json
{
  "type": "unary_comparison_operator",
  "operator": "is_null",
  "column": {
    "name": "title"
  }
}
```

### Binary Operators

Binary comparison operators are denoted by expressions with a `type` field of `binary_comparison_operator`.

The set of available operators depends on the type of the column involved in the expression. The `operator` property should specify the name of one of the binary operators from the field's [scalar type](../schema/scalar-types.md) definition.

The type [`ComparisonValue`](../../reference/types.md#comparisonvalue) describes the valid inhabitants of the `value` field. The `value` field should be an expression which evaluates to a value whose type is compatible with the definition of the comparison operator.

#### Equality Operators

This example makes use of an `eq` operator, which is defined using the `equal` semantics, to test a single column for equality with a scalar value:

```json
{{#include ../../../../ndc-reference/tests/query/predicate_with_eq/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/predicate_with_eq/request.json:3: }}
```

#### Set Membership Operators

This example uses an `in` operator, which is defined using the `in` semantics, to test a single column for membership in a set of values:

```json
{{#include ../../../../ndc-reference/tests/query/predicate_with_in/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/predicate_with_in/request.json:3: }}
```

#### Custom Operators

This example uses a custom `like` operator:

```json
{{#include ../../../../ndc-reference/tests/query/predicate_with_like/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/predicate_with_like/request.json:3: }}
```

### Nested Array Comparison Operators

If the connector declares support for the `query.nested_fields.filter_by.nested_arrays` capability, it can receive expressions of type `array_comparison`. These expressions allow scalar array-specific comparisons against columns that contain an array of scalar values.

There are two supported comparison operators that connectors can declare support for:

- `contains`: Whether or not the array contains the specified scalar value. This must be supported for all types that can be contained in an array that implement an 'eq' comparison operator.
  - Capability: `query.nested_fields.filter_by.nested_arrays.contains`
- `is_empty`: Whether or not the array is empty. This must be supported no matter what type is contained in the array.
  - Capability: `query.nested_fields.filter_by.nested_arrays.is_empty`

This example finds `institutions` where the nested `location.campuses` array contains the `Lindholmen` value:

```json
{{#include ../../../../ndc-reference/tests/query/predicate_with_array_contains/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/predicate_with_array_contains/request.json:3: }}
```

This example finds `countries` which have an empty `cities` array:

```json
{{#include ../../../../ndc-reference/tests/query/predicate_with_array_is_empty/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/predicate_with_array_is_empty/request.json:3: }}
```

### Columns in Operators

Comparison operators compare values. The value on the left hand side of any operator is described by a [`ComparisonTarget`](../../reference/types.md#comparisontarget), and the various cases will be explained next.

#### Referencing a column from the same collection

If the `ComparisonTarget` has type `column`, then the `name` property refers to a column in the current collection. The `arguments` property allows clients to submit argument values for columns that require [arguments](./arguments.html#field-arguments).

#### Referencing nested fields within columns

If the `field_path` property is empty or not present then the target is the value of the named column.

If `field_path` is non-empty then it refers to a path to a nested field within the named column

_Note_: a `ComparisonTarget` may only have a non-empty `field_path` if the connector supports capability `query.nested_fields.filter_by`.

#### Computing an aggregate

If the `ComparisonTarget` has type `aggregate`, then the target is an aggregate computed over a related collection. The relationship is described by the (non-empty) `path` field, and the aggregate to compute is specified in the `aggregate` field.

For example, this query finds authors who have written exactly 2 articles:

```json
{{#include ../../../../ndc-reference/tests/query/predicate_with_star_count/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/predicate_with_star_count/request.json:3: }}
```

_Note_: type `aggregate` will only be sent if the `query.aggregates.filter_by` capability is turned on. If that capability is turned on, then the schema response should also contain the `capabilities.query.aggregates.filter_by` object. That object should indicate the scalar type used for the result type of count aggregates (`star_count` and `column_count`), so that clients can know what comparison operators are valid.

### Values in Binary Operators

Binary (including array-valued) operators compare columns to _values_, but there are several types of valid values:

- Scalar values, as seen in the examples above, compare the column to a specific value,
- Variable values compare the column to the current value of a [variable](./variables.md),
- Column values compare the column to _another_ column.

#### Referencing a column from a collection in scope

When an expression appears inside one or more [exists expressions](#exists-expressions), there are multiple collections in scope.

If the `query.exists.named_scopes` capability is enabled then these scopes can be named explicitly when referencing a column in an outer scope. The `scope` field of the `ComparisonValue` type can be used to specify the scope of a column reference.

Scopes are named by integers in the following manner:

- The scope named `0` refers to the current collection,
- The scope named `1` refers to the collection under consideration outside the immediately-enclosing exists expression.
- Scopes `2`, `3`, and so on, refer to the collections considered during the evaluation of expressions outside subsequently enclosing exists expressions.

Therefore, the largest valid scope is the maximum nesting depth of exists expressions, up to the nearest enclosing `Query` object.

Put another way, we can consider a stack of scopes which grows as we descend into each nested exists expression. Each stack frame contains the collection currently under consideration. The named scopes are then the top-down indices of elements of this stack.

For example, we can express an equality between an `author_id` column and the `id` column of the enclosing `author` object (in scope `1`):

```json
{{#include ../../../../ndc-reference/tests/query/named_scopes/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/named_scopes/request.json:3: }}
```

## `EXISTS` expressions

An `EXISTS` expression tests whether a row exists in some possibly-related collection, and is denoted by an expression with a `type` field of `exists`.

`EXISTS` expressions can query related or unrelated collections.

### Related Collections

Related collections are related to the original collection by a relationship in the `collection_relationships` field of the top-level [`QueryRequest`](../../reference/types.md#queryrequest).

For example, this query fetches authors who have written articles whose titles contain the string `"Functional"`:

```json
{{#include ../../../../ndc-reference/tests/query/predicate_with_exists/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/predicate_with_exists/request.json:3: }}
```

#### Nested relationships

If the related collection is related from a field inside a nested object, then the field path to the nested object can be first descended through using `field_path` before the relationship is navigated.

Only connectors that enable the `relationships.nested` capability will receive these sorts of queries.

In this example, the relationship joins from the nested `location.country_id` across to the `id` column on the `countries` collection.

```json
{{#include ../../../../ndc-reference/tests/query/predicate_with_exists_from_nested_field/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/predicate_with_exists_from_nested_field/request.json:3: }}
```

### Unrelated Collections

If the `query.exists.unrelated` capability is enabled, then exists expressions can reference unrelated collections.

Unrelated exists expressions can be useful when using collections with [arguments](./arguments.md). For example, this query uses the unrelated `author_articles` collection, providing its arguments via the source row's columns:

```json
{{#include ../../../../ndc-reference/tests/query/table_argument_unrelated_exists/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/table_argument_unrelated_exists/request.json:3: }}
```

It can also be useful to [reference a column in another scope](#referencing-a-column-from-a-collection-in-scope) when using unrelated exists expressions.

### Nested Collections

If the `query.exists.nested_collections` capability is enabled, then exists expressions can reference [nested collections](./field-selection.md#nested-collections).

For example, this query finds `institutions` which employ at least one staff member whose last name contains the letter `s`:

```json
{{#include ../../../../ndc-reference/tests/query/predicate_with_exists_in_nested_collection/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/predicate_with_exists_in_nested_collection/request.json:3: }}
```

[References to columns in another scope](#referencing-a-column-from-a-collection-in-scope) may be useful when using these sorts of expressions, in order to refer to columns from the outer (unnested) row.

### Nested Scalar Collections

If the `query.exists.nested_scalar_collections` capability is enabled, then exists expressions can reference columns that contain nested arrays of scalar values. In this case, each element of the nested array is lifted into a virtual row with the element value in a field called `__value`. This allows predicate applied to the exists to reference the `__value` column to compare against the scalar element.

For example, if there was a nested array such as `[1,2,3]`, it would be converted into a virtual rows `[{"__value": 1}, {"_value": 2}, {"_value": 3}]`.

For example, this query finds `institutions` that have at least one campus whose name contains the letter `d` (campuses are a string array nested inside location):

```json
{{#include ../../../../ndc-reference/tests/query/predicate_with_exists_in_nested_scalar_collection/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/predicate_with_exists_in_nested_scalar_collection/request.json:3: }}
```

## Conjunction of expressions

To express the conjunction of multiple expressions, specify a `type` field of `and`, and provide the expressions in the `expressions` field.

For example, to test if the `first_name` column is null _and_ the `last_name` column is also null:

```json
{
  "type": "and",
  "expressions": [
    {
      "type": "unary_comparison_operator",
      "operator": "is_null",
      "column": {
        "name": "first_name"
      }
    },
    {
      "type": "unary_comparison_operator",
      "operator": "is_null",
      "column": {
        "name": "last_name"
      }
    }
  ]
}
```

## Disjunction of expressions

To express the disjunction of multiple expressions, specify a `type` field of `or`, and provide the expressions in the `expressions` field.

For example, to test if the `first_name` column is null _or_ the `last_name` column is also null:

```json
{
  "type": "or",
  "expressions": [
    {
      "type": "unary_comparison_operator",
      "operator": "is_null",
      "column": {
        "name": "first_name"
      }
    },
    {
      "type": "unary_comparison_operator",
      "operator": "is_null",
      "column": {
        "name": "last_name"
      }
    }
  ]
}
```

## Negation

To express the negation of an expressions, specify a `type` field of `not`, and provide that expression in the `expression` field.

For example, to test if the `first_name` column is _not_ null:

```json
{
  "type": "not",
  "expression": {
    "type": "unary_comparison_operator",
    "operator": "is_null",
    "column": {
      "name": "first_name"
    }
  }
}
```

## See also

- Type [`Expression`](../../reference/types.md#expression)
