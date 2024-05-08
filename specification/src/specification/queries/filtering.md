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

### Columns in Operators

Comparison operators compare columns to values. The column on the left hand side of any operator is described by a [`ComparisonTarget`](../../reference/types.md#comparisontarget), and the various cases will be explained next.

#### Referencing a column from the same collection

If the `ComparisonTarget` has type `column`, and the `path` property is empty, then the `name` property refers to a column in the current collection.

#### Referencing a column from a related collection

If the `ComparisonTarget` has type `column`, and the `path` property is non-empty, then the `name` property refers to column in a related collection. The path consists of a collection of [`PathElement`](../../reference/types.md#pathelement)s, each of which references a named [relationship](./relationships.md), any [collection arguments](./arguments.md), and a [predicate expression](./filtering.md) to be applied to any relevant rows in the related collection.

When a `PathElement` references an _array_ relationship, the enclosing operator should be considered _existentially quantified_ over all related rows.

#### Referencing a column from the root collection

If the `ComparisonTarget` has type `root_collection_column`, then the `name` property refers to a column in the _root collection_.

The root collection is defined as the collection in scope at the nearest enclosing [`Query`](../../reference/types.md#query), and the column should be chosen from the _row_ in that collection which was in scope when that `Query` was being evaluated.

#### Referencing nested fields within columns

If the `field_path` property is empty or not present then the target is the value of the named column.
If `field_path` is non-empty then it refers to a path to a nested field within the named column.
(A `ComparisonTarget` may only have a non-empty `field_path` if the connector supports capability `query.nested_fields.filter_by`.)

### Values in Binary Operators

Binary (including array-valued) operators compare columns to _values_, but there are several types of valid values:

- Scalar values, as seen in the examples above, compare the column to a specific value,
- Variable values compare the column to the current value of a [variable](./variables.md),
- Column values compare the column to _another_ column, possibly selected from a different collection. Column values are also described by a [`ComparisonTarget`](../../reference/types.md#comparisontarget).

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

### Unrelated Collections

```json
{{#include ../../../../ndc-reference/tests/query/predicate_with_unrelated_exists/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/predicate_with_unrelated_exists/request.json:3: }}
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