# Scalar Types

The schema should describe any irreducible _scalar types_. Scalar types can be used as the types of columns, or in general as the types of object fields.

Scalar types define several types of operations, which extend the capabilities of the query and mutation APIs: _comparison operators_ and _aggregate functions_.

## Type Representations

A scalar type definition must include a _type representation_. The representation indicates to potential callers what values can be expected in responses, and what values are considered acceptable in requests.

### Supported Representations

| `type`        | Description                                                                           | JSON representation |
| ------------- | ------------------------------------------------------------------------------------- | ------------------- |
| `boolean`     | Boolean                                                                               | Boolean             |
| `string`      | String                                                                                | String              |
| `int8`        | An 8-bit signed integer with a minimum value of -2^7 and a maximum value of 2^7 - 1   | Number              |
| `int16`       | A 16-bit signed integer with a minimum value of -2^15 and a maximum value of 2^15 - 1 | Number              |
| `int32`       | A 32-bit signed integer with a minimum value of -2^31 and a maximum value of 2^31 - 1 | Number              |
| `int64`       | A 64-bit signed integer with a minimum value of -2^63 and a maximum value of 2^63 - 1 | String              |
| `float32`     | An IEEE-754 single-precision floating-point number                                    | Number              |
| `float64`     | An IEEE-754 double-precision floating-point number                                    | Number              |
| `biginteger`  | Arbitrary-precision integer string                                                    | String              |
| `bigdecimal`  | Arbitrary-precision decimal string                                                    | String              |
| `uuid`        | UUID string (8-4-4-4-12 format)                                                       | String              |
| `date`        | ISO 8601 date                                                                         | String              |
| `timestamp`   | ISO 8601 timestamp                                                                    | String              |
| `timestamptz` | ISO 8601 timestamp-with-timezone                                                      | String              |
| `geography`   | GeoJSON, per RFC 7946                                                                 | JSON                |
| `geometry`    | GeoJSON Geometry object, per RFC 7946                                                 | JSON                |
| `bytes`       | Base64-encoded bytes                                                                  | String              |
| `json`        | Arbitrary JSON                                                                        | JSON                |

### Enum Representations

A scalar type with a representation of type `enum` accepts one of a set of string values, specified by the `one_of` argument.

For example, this representation indicates that the only three valid values are the strings `"foo"`, `"bar"` and `"baz"`:

```json
{
  "type": "enum",
  "one_of": ["foo", "bar", "baz"]
}
```

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

##### Note: syntactic equality

Specifically, a predicate expression which uses an operator of type `equal` should implement _syntactic equality_:

- An expression which tests for equality of a column with a _scalar_ value or _variable_ should return that scalar value exactly (equal as JSON values) for all rows in each corresponding row set, whenever the same column is selected.
- An expression which tests for equality of a column with _another column_ should return the same values in both columns (equal as JSON values) for all rows in each corresponding row set, whenever both of those those columns are selected.

This type of equality is quite strict, and it might not be possible to implement such an operator for all scalar types. For example, a case-insensitive string type's natural case-insensitive equality operator would not meet the criteria above. In such cases, the scalar type should _not_ provide an _equal_ operator.

#### `In`

An operator defined using type `in` tests if a column value is a member of an array of values. The array is specified either as a scalar, a variable, or as the value of another column.

It should accept an array type as its argument, whose element type is the scalar type for which it is defined. It should be equivalent to a disjunction of individual equality tests on the elements of the provided array, where the equality test is an equivalence relation in the same sense as above.

#### `less_than`, `greater_than`, `less_than_or_equal`, `greater_than_or_equal`

An operator defined using type `less_than` tests if a column value is less than a specified value. Similarly for the other comparisons here.

If a connector defines more than one of these standard operators, then they should be compatible:

- When using `less_than`, a row should be included in the generated row set if and only if it would _not_ be returned in the corresponding `greater_than_or_equal` comparison, and vice versa. More succinctly, it is expected that `x < y` holds exactly when `x >= y` does not hold.
- It is expected that `x < y` holds exactly when `y > x` holds.
- It is expected that `x <= y` holds exactly when `y >= x` holds.

The `less_than_or_equal` and `greater_than_or_equal` operators are expected to be _reflexive_. That is, they should return a superset of those rows returned by the corresponding `equal` (syntactic equality) operator.

Each of these four operators is expected to be _transitive_. That is, for example `x < y` and `y < z` together imply `x < z`, and similarly for the other operators.

#### `contains`, `icontains`, `starts_with`, `istarts_with`, `ends_with`, `iends_with`

These operators must only apply to scalar types whose type representation is `string`.

An operator defined using type `contains` tests if a string-valued column on the left contains a string value on the right. `icontains` is the case-insensitive variant.

An operator defined using type `starts_with` tests if a string-valued column on the left starts with a string value on the right. `istarts_with` is the case-insensitive variant.

An operator defined using type `ends_with` tests if a string-valued column on the left ends with a string value on the right. `iends_with` is the case-insensitive variant.

### Custom Comparison Operators

Data connectors can also define custom comparison operators using type `custom`. A custom operator is defined by its argument type, and its semantics is undefined.

## Aggregate Functions

Aggregate functions extend the query AST with the ability to express new aggregates within the `aggregates` portion of a query. They also allow sorting the query results via the `order_by` query field.

_Note_: data connectors are required to implement the _count_ and _count-distinct_ aggregations for columns of all scalar types, and those operator is distinguished in the query AST. There is no need to define these aggregates as aggregate functions.

For example, a data connector might augment a `Float` scalar type with a `SUM` function which aggregates a sum of a collection of floating-point numbers.

Just like for comparison operators, an aggregate function is either a _standard_ function, or a custom function.

To define an aggregate function, add a [`AggregateFunctionDefinition`](../../reference/types.md#aggregatefunctiondefinition) to the `aggregate_functions` field of the schema response.

For example:

```json
{
  "scalar_types": {
    "Float": {
      "aggregate_functions": {
        "sum": {
          "type": "sum",
          "result_type": "Float"
        },
        "stddev": {
          "type": "custom",
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

### Standard Aggregate Functions

#### `sum`

An aggregate function defined using type `sum` should return the numerical sum of its provided values.

The result type should be provided explicitly, in the `result_type` field, and should be a scalar type with a type representation of either `Int64` or `Float64`, depending on whether the scalar type defining this function has an integer representation or floating point representation.

A `sum` function should ignore the order of its input values, and should be invariant of partitioning, that is: `sum(x, sum(y, z))` = `sum(x, y, z)` for any partitioning `x, y, z` of the input values. It should return `0` for an empty set of input values.

#### `average`

An aggregate function defined using type `average` should return the average of its provided values.

The result type should be provided explicitly, in the `result_type` field, and should be a scalar type with a type representation of `Float64`.

An `average` function should ignore the order of its input values. It should return `null` for an empty set of input values.

#### `min`, `max`

An aggregate function defined using type `min` or `max` should return the minimal/maximal value from its provided values, according to some ordering.

Its implicit result type, i.e. the type of the aggregated values, is the same as the scalar type on which the function is defined, but with nulls allowed if not allowed already.

A `min`/`max` function should return null for an empty set of input values.

If the set of input values is a singleton, then the function should return the single value.

A `min`/`max` function should ignore the order of its input values, and should be invariant of partitioning, that is: `min(x, min(y, z))` = `min(x, y, z)` for any partitioning `x, y, z` of the input values.

### Custom Aggregate Functions

A custom aggregate function has type `custom` and is defined by its _result type_ - that is, the type of the aggregated data. The result type can be any type, not just a scalar type.

## See also

- Type [`ScalarType`](../../reference/types.md#scalartype)
- [`Filtering`](../queries/filtering.md)
- [`Sorting`](../queries/sorting.md)
- [`Aggregates`](../queries/aggregates.md)
