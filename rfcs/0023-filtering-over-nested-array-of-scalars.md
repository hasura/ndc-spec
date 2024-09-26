# Filtering over nested arrays of scalars

## Purpose

Right now, if you have an object type defined that has a field that is an array of a scalar type, there is no way to filter a collection of the object type by whether or not something exists inside that field's array of scalar type.

Imagine we have a `Users` collection of `User` object type:

```yaml
User:
  fields:
    id:
      type:
        type: named
        name: Int
    name:
      type:
        type: named
        name: String
    roles:
      type:
        type: array
        element_type:
          type: named
          name: String
```

An example object might be:

```yaml
id: 1
name: Daniel
roles: ["admin", "user"]
```

We might want to issue a query where we filter the `Users` collection so we only return `User` objects where `roles` contains the string `"admin"`. We might try to do this using the following NDC query:

```yaml
collection: Users
query:
  fields:
    id:
      type: column
      column: id
  predicate:
    type: binary_comparison_operator
    column:
      type: column
      name: roles
      path: []
    operator: eq
    value:
      type: scalar
      value: admin
arguments: {}
collection_relationships: {}
```

However, this would currently be illegal since the type of the the `roles` column (array of `String`) does not match the value's type (`String`).

## Proposal
We could add another variant to Expression to represent a comparison against an array type:

```rust
pub enum Expression {
    ...
    ArrayComparison {
        column: ComparisonTarget,
        comparison: ArrayComparison,
    },
}
```

The `ArrayComparison` type would then capture the different types of comparisons one could do against the array:

```rust
pub enum ArrayComparison {
    /// Perform a binary comparison operation against the elements of the array.
    /// The comparison is asserting that there must exist at least one element 
    /// in the array that the comparison succeeds for
    ExistsBinary {
        operator: ComparisonOperatorName,
        value: ComparisonValue,
    },
    /// Perform a unary comparison operation against the elements of the array.
    /// The comparison is asserting that there must exist at least one element 
    /// in the array that the comparison succeeds for
    ExistsUnary {
        operator: UnaryComparisonOperator
    },
    /// Nest a comparison through one level of a nested array, asserting that
    /// there must exist at least one element in the outer array who matches
    /// the comparison applied to the inner array
    ExistsInNestedArray {
        nested_comparison: Box<ArrayComparison>
    },
    /// Check if the array contains the specified value
    Contains {
        value: ComparisonValue,
    },
    /// Check is the array is empty
    IsEmpty,
}
```

Whether or not these new array comparisons would be supported by the connector would be declared in the capabilities:

```jsonc
{
  "query": {
    "aggregates": {},
    "variables": {},
    "nested_fields": {
      "filter_by": {
        // NEW!!
        // Does the connector support filtering over nested arrays
        "nested_arrays": {
          // Does the connector support filtering over nested arrays using existential quantification.
          // This must be supported for all types that can be contained in an array that have a comparison operator.
          "exists": {
            // Does the connector support filtering over nested arrays of arrays using existential quantification
            "nested": {}
          },
          // Does the connector support filtering over nested arrays by checking if the array contains a value.
          // This must be supported for all types that can be contained in an array.
          "contains": {},
          // Does the connector support filtering over nested arrays by checking if the array is empty.
          // This must be supported no matter what type is contained in the array.
          "isEmpty": {}
        } 
      },
      "order_by": {},
      "aggregates": {}
    },
    "exists": {
      "nested_collections": {}
    }
  },
  "mutation": {},
  "relationships": {
    "relation_comparisons": {},
    "order_by_aggregate": {}
  }
}
```

## Alternative Proposal

We could update the definition of `ComparisonTarget::Column` to specify that if the targeted column is an array of scalars, then the comparison operator should be considered to be existentially quantified over all elements in the array. In simpler terms, at least one element in the array of scalars must match the specified comparison.

This behaviour for `ComparisonTarget::Column` is new, and as such would need to be gated behind a new capability so that existing connectors would not receive queries expecting this behaviour.

```json
{
  "query": {
    "aggregates": {},
    "variables": {},
    "nested_fields": {
      "filter_by": {
        "scalar_arrays": {} // NEW!!
      },
      "order_by": {},
      "aggregates": {}
    },
    "exists": {
      "nested_collections": {}
    }
  },
  "mutation": {},
  "relationships": {
    "relation_comparisons": {},
    "order_by_aggregate": {}
  }
}
```

### Issues

#### Implicit existential quantification

This new interpretation of the query structure is implicit, which is suboptimal as it may be non-obvious to connector authors that this is how things are supposed to work. It is better to be explicit with such things.

It also disallows direct comparison of a complex type to a literal value of that complex type (something that isn't supported right now, anyway). For example, this is now inexpressible due to the implicit existential quantification:

```yaml
collection: Users
query:
  fields:
    id:
      type: column
      column: id
  predicate:
    type: binary_comparison_operator
    column:
      type: column
      name: roles
      path: []
    operator: eq
    value:
      type: scalar
      value: ["admin", "users"] # The roles must be exactly admin and users, in that order
arguments: {}
collection_relationships: {}
```

A way that _explicit_ existential quantification could be represented could be to add a new variant to `ComparisonTarget`, `ExistsInColumn`:

```rust
pub enum ComparisonTarget {
    Column {
        /// The name of the column
        name: FieldName,
        /// Path to a nested field within an object column
        field_path: Option<Vec<FieldName>>,
    },
    ExistsInColumn {
        /// The name of the column
        name: FieldName,
        /// Path to a nested field within an object column
        field_path: Option<Vec<FieldName>>,
    },
    Aggregate {
        /// The aggregation method to use
        aggregate: Aggregate,
        /// Non-empty collection of relationships to traverse
        path: Vec<PathElement>,
    },
}
```

Then you could write a query more explicitly like so:

```yaml
collection: Users
query:
  fields:
    id:
      type: column
      column: id
  predicate:
    type: binary_comparison_operator
    column:
      type: exists_in_column # New!
      name: roles
      path: []
    operator: eq
    value:
      type: scalar
      value: admin
arguments: {}
collection_relationships: {}
```

The use of `ComparisonTarget::ExistsInColumn` would be gated behind the proposed capability.

The issue with this is that it requires more work to support, as more extensive changes are required to v3-engine so that it uses this new `ComparisonTarget`.

#### How about existential quantification over arrays of nested objects?

What about if we had the following `User` and `Role` object types:

```yaml
User:
  fields:
    id:
      type:
        type: named
        name: Int
    name:
      type:
        type: named
        name: String
    roles:
      type:
        type: array
        element_type:
          type: named
          name: Role

Role:
  fields:
    name:
      type:
        type: named
        name: String
    assignedAt:
      type:
        type: named
        name: DateTime
```

An example object might be:

```yaml
id: 1
name: Daniel
roles:
  - name: admin
    assignedAt: 2024-09-25T14:51:00Z
  - name: user
    assignedAt: 2024-09-25T12:14:00Z
```

Could we write a query that filtered by the `name` property in the nested array of `Role` object types like so, thanks to the implicit existential quantification?

```yaml
collection: Users
query:
  fields:
    id:
      type: column
      column: id
  predicate:
    type: binary_comparison_operator
    column:
      type: column
      name: roles
      field_path: [name] # Navigate into the name property of the Role object
      path: []
    operator: eq
    value:
      type: scalar
      value: admin
arguments: {}
collection_relationships: {}
```

This is inadvisable to allow, and such a query can already be expressed using explicit nested collection `Expression::Exists` queries, like so:

```yaml
collection: Users
query:
  fields:
    id:
      type: column
      column: id
  predicate:
    type: exists
      in_collection:
        type: nested_collection
        column: roles
      predicate:
        type: binary_comparison_operator
        column:
          type: column
          name: name
          path: []
        operator: eq
        value:
          type: scalar
          value: admin
arguments: {}
collection_relationships: {}
```

We should state that the existential quantification only works when the _end-point_ of the `ComparisonTarget::Column` is targeting an array of scalars. `field_path` can only be used to navigate nested objects.
