# Types

Several definitions in this specification make mention of _types_. Types are used to categorize the sorts of data returned and accepted by a data connector.

Scalar and named object types are defined in the [schema response
](./schema/README.md), and referred to by name at the point of use.

Array types, nullable types and predicate types are constructed at the point of use.

## Named Types

To refer to a named (scalar or object) type, use the type `named`, and provide the name:

```json
{
  "type": "named",
  "name": "String"
}
```

## Array Types

To refer to an array type, use the type `array`, and refer to the type of the elements of the array in the `element_type` field:

```json
{
  "type": "array",
  "element_type": {
    "type": "named",
    "name": "String"
  }
}
```

## Nullable Types

To refer to a nullable type, use the type `nullable`, and refer to the type of the underlying (non-null) inhabitants in the `underlying_type` field:

```json
{
  "type": "nullable",
  "underlying_type": {
    "type": "named",
    "name": "String"
  }
}
```

Nullable and array types can be nested. For example, to refer to a nullable array of nullable strings:

```json
{
  "type": "nullable",
  "underlying_type": {
    "type": "array",
    "element_type": {
      "type": "nullable",
      "underlying_type": {
        "type": "named",
        "name": "String"
      }
    }
  }
}
```

## Predicate Types

A predicate type can be used to represent valid predicates (of type [`Expression`](../reference/types.md#expression)) for an object type. A value of a predicate type is represented, in inputs and return values, as a JSON value which parses as an `Expression`. Valid expressions are those which refer to the columns of the object type. 

To refer to a predicate type, use the type `predicate`, and provide the name of the object type:

```json
{
  "type": "predicate",
  "object_type_name": "article"
}
```

Note: predicate types are intended primarily for use in [arguments](./queries/arguments.md) to functions and [procedures](./mutations/procedures.md), but they can be used anywhere a [`Type`](../reference/types.md) is expected, including in output types.

## See also

- Type [`Type`](../reference/types.md#type)
- [Scalar types](./schema/scalar-types.md)
- [Object types](./schema/object-types.md)