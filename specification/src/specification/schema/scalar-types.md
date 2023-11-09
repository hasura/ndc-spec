# Scalar Types

The schema should describe any irreducible _scalar types_. Scalar types can be used as the types of columns, or in general as the types of object fields.

Scalar types define several types of operations, which extend the capabilities of the query and mutation APIs: _comparison operators_, _aggregation functions_, and _update operators_.

## Comparison Operators

Comparison operators extend the query AST with the ability to express new binary comparison expressions in the predicate.

_Note_: data connectors are required to implement the _equality_ operator for all scalar types, and that operator is distinguished in the query AST. There is no need to define the equality operator as a comparison operator.

For example, a data connector might augment a `String` scalar type with a `LIKE` operator which tests for a fuzzy match based on a regular expression.

A comparison operator is defined by its _argument type_ - that is, the type of the right hand side of the binary operator it represents.

To define a comparison operator, add a [`ComparisonOperatorDefinition`](../../reference/types.md#comparisonoperatordefinition) to the `comparison_operators` field of the schema response.

For example:

```json
{
  "scalar_types": {
    "String": {
      "aggregate_functions": {},
      "comparison_operators": {
        "like": {
          "argument_type": {
            "type": "named",
            "name": "String"
          }
        }
      },
      "update_operators": {}
    }
  },
  ...
}
```

## Aggregation Functions

Aggregation functions extend the query AST with the ability to express new aggregates within the `aggregates` portion of a query.

_Note_: data connectors are required to implement the _count_ and _count-distinct_ aggregations for columns of all scalar types, and those operator is distinguished in the query AST. There is no need to define these aggregates as aggregation functions.

For example, a data connector might augment a `Float` scalar type with a `SUM` function which aggregates a sum of a collection of floating-point numbers.

An aggregation function is defined by its _result type_ - that is, the type of the aggregated data.

To define an aggregation function, add a [`AggregateFunctionDefinition`](../../reference/types.md#aggregatefunctiondefinition) to the `aggregate_functions` field of the schema response.

For example:

```json
{
  "scalar_types": {
    "Float": {
      "aggregate_functions": {
        "sum": {
          "result_type": {
            "type": "named",
            "name": "Float"
          }
        }
      },
      "comparison_operators": {},
      "update_operators": {}
    }
  },
  ...
}
```

## Update Operators

Update operators extend the mutation AST with the ability to express new kinds of updates on individual columns with those types.

_Note_: data connectors are required to implement the _set_ update operator for columns of all scalar types, and that operator is distinguished in the mutation AST. There is no need to define the set operator as an update operator.

For example, a data connector might augment an `Int` scalar type with an `INC` operator which increments the value of an integer-valued column.

An update operator is defined by its _argument type_ - that is, the type of the right hand side of the binary operator it represents.

To define an update operator, add a [`UpdateOperatorDefinition`](../../reference/types.md#updateoperatordefinition) to the `update_operators` field of the schema response.

For example:

```json
{
  "scalar_types": {
    "Int": {
      "aggregate_functions": {},
      "comparison_operators": {},
      "update_operators": {
        "inc": {
          "argument_type": {
            "type": "named",
            "name": "Int"
          }
        }
      }
    }
  },
  ...
}
```

## See also

- Type [`ScalarType`](../../reference/types.md#scalartype)
- [`Filtering`](../queries/filtering.md)
- [`Aggregates`](../queries/aggregates.md)
- [`Update Mutations`](../mutations/update.md)
