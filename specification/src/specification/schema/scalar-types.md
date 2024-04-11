# Scalar Types

The schema should describe any irreducible _scalar types_. Scalar types can be used as the types of columns, or in general as the types of object fields.

Scalar types define several types of operations, which extend the capabilities of the query and mutation APIs: _comparison operators_ and _aggregation functions_.

## Type Representations

A scalar type definition can include an optional _type representation_. The representation, if provided, indicates to potential callers what values can be expected in responses, and what values are considered acceptable in requests.

If the representation is omitted, it defaults to `json`.

### Supported Representations

| `type` | Description | JSON representation |
| - | - | - |
| `boolean` | Boolean | Boolean |
| `string` | String | String |
| `int8` | An 8-bit signed integer with a minimum value of -2^7 and a maximum value of 2^7 - 1 | Number |
| `int16` | A 16-bit signed integer with a minimum value of -2^15 and a maximum value of 2^15 - 1 | Number |
| `int32` | A 32-bit signed integer with a minimum value of -2^31 and a maximum value of 2^31 - 1 | Number |
| `int64` | A 64-bit signed integer with a minimum value of -2^63 and a maximum value of 2^63 - 1 | String |
| `float32` | An IEEE-754 single-precision floating-point number | Number |
| `float64` | An IEEE-754 double-precision floating-point number | Number |
| `biginteger` | Arbitrary-precision integer string | String |
| `bigdecimal` | Arbitrary-precision decimal string | String |
| `uuid` | UUID string (8-4-4-4-12 format) | String |
| `date` | ISO 8601 date | String |
| `timestamp` | ISO 8601 timestamp | String |
| `timestamptz` | ISO 8601 timestamp-with-timezone | String |
| `geography` | GeoJSON, per RFC 7946 | JSON |
| `geometry` | GeoJSON Geometry object, per RFC 7946 | JSON |
| `bytes` | Base64-encoded bytes | String |
| `json` | Arbitrary JSON | JSON |

### Enum Representations

A scalar type with a representation of type `enum` accepts one of a set of string values, specified by the `one_of` argument.

For example, this representation indicates that the only three valid values are the strings `"foo"`, `"bar"` and `"baz"`:

```json
{
  "type": "enum",
  "one_of": ["foo", "bar", "baz"]
}
```

### Deprecated Representations

The following representations are deprecated as of version 0.1.2:

| `type` | Description | JSON representation |
| - | - | - |
| `number` | Any JSON number | Number |
| `integer` | Any JSON number with no decimal part | Number |

Connectors should use the sized integer and floating-point types instead.

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
