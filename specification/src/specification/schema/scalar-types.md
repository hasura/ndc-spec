# Scalar Types

The schema should describe any irreducible _scalar types_. Scalar types can be used as the types of columns, or in general as the types of object fields.

Scalar types define several types of operations, which extend the capabilities of the query and mutation APIs: _comparison operators_ and _aggregation functions_.

## Comparison Operators

Comparison operators extend the query AST with the ability to express new binary comparison expressions in the predicate.

For example, a data connector might augment a `String` scalar type with a `LIKE` operator which tests for a fuzzy match based on a regular expression.

A comparison operator is either a _standard_ operator, or a custom operator.

To define a comparison operator, add a [`ComparisonOperatorDefinition`](../../reference/types.md#comparisonoperatordefinition) to the `comparison_operators` field of the schema response.

For example:

```json
{
  "scalar_types": {
    "String": {
      "aggregate_functions": {},
      "comparison_operators": {
        "like": {
          "type": "custom",
          "argument_type": {
            "type": "named",
            "name": "String"
          }
        }
      }
    }
  },
  ...
}
```

### Standard Comparison Operators

#### `Equal`

An operator defined using type `equal` tests if a column value is equal to a scalar value, another column value, or a variable.

The precise semantics of this equality operator are not specified, but the operator should define an equivalence relation. That is, its argument type should be the same scalar type for which it is defined, and at the level of values, it should be reflexive, symmetric and transitive.

For example, an equality operator on a string type might test equality at the level of the underlying bytes, or might perform a coarser-grained test such as case-insensitive comparison.

#### `In`

An operator defined using type `in` tests if a column value is a member of an array of values. The array is specified either as a scalar, a variable, or as the value of another column.

It should accept an array type as its argument, whose element type is the scalar type for which it is defined. It should be equivalent to a disjunction of individual equality tests on the elements of the provided array, where the equality test is an equivalence relation in the same sense as above.

### Custom Comparison Operators

Data connectors can also define custom comparison operators using type `custom`. A custom operator is defined by its argument type, and its semantics is undefined.

## Aggregation Functions

Aggregation functions extend the query AST with the ability to express new aggregates within the `aggregates` portion of a query. They also allow sorting the query results via the `order_by` query field.

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
      "comparison_operators": {}
    }
  },
  ...
}
```

## See also

- Type [`ScalarType`](../../reference/types.md#scalartype)
- [`Filtering`](../queries/filtering.md)
- [`Sorting`](../queries/sorting.md)
- [`Aggregates`](../queries/aggregates.md)
