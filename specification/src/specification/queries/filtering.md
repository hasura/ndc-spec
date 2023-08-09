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

The set of available operators depends on the type of the column involved in the expression. The `equal` operator should be implemented for all types of columns. 

See type [`BinaryComparisonOperator`](../../reference/types.md#binarycomparisonoperator).

#### `equals`

`equals` tests if a column value is equal to a scalar value, another column value, or a variable.

See type [`ComparisonValue`](../../reference/types.md#comparisonvalue) for the valid inhabitants of the `value` field.

```json
{
    "type": "binary_comparison_operator",
    "operator": {
        "type": "equal"
    },
    "column": {
        "name": "title"
    },
    "value": {
        "type": "scalar",
        "value": "The Next 700 Programming Languages"
    }
}
```

### Custom Binary Comparison Operators

Data connectors can also extend the expression grammar by defining comparison operators on each [scalar type](../schema/scalar-types.md) in the schema response.

For example, here is an expression which uses a custom `like` operator provided on the `String` type in the reference implementation:

```json
{
    "type": "binary_comparison_operator",
    "operator": {
        "type": "other",
        "name": "like"
    },
    "column": {
        "name": "title"
    },
    "value": {
        "type": "scalar",
        "value": "^.*Functional Programming.*$"
    }
}
```

### Binary Array-Valued Comparison Operators

Binary comparison operators are denoted by expressions with a `type` field of `binary_array_comparison_operator`. 

#### `in`

`in` tests if a column value is a member of an array of values, each of which can be a scalar value, another column value, or a variable.

See type [`ComparisonValue`](../../reference/types.md#comparisonvalue) for the valid inhabitants of the `value` field.

```json
{
    "type": "binary_array_comparison_operator",
    "operator": "in",
    "column": {
        "name": "id"
    },
    "values": [
        {
            "type": "scalar",
            "value": "1"
        },
        {
            "type": "scalar",
            "value": "2"
        }
    ]
}
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