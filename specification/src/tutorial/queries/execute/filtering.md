# Filtering

The next step is to filter the rows based on the provided predicate expression:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:execute_query_filter}}
```

As we can see, the function delegates to the `eval_expression` function in order to evaluate the predicate on each row.

## Evaluating expressions

The `eval_expression` function evaluates a predicate by pattern matching on the type of the expression `expr`, and returns a boolean value indicating whether the current row matches the predicate:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_signature}}
```

### Logical expressions

The first category of expression types are the _logical expressions_ - _and_ (conjunction), _or_ (disjunction) and _not_ (negation) - whose evaluators are straightforward:

- To evaluate a conjunction/disjunction of subexpressions, we evaluate all of the subexpressions to booleans, and find the conjunction/disjunction of those boolean values respectively.
- To evaluate the negation of a subexpression, we evaluate the subexpression to a boolean value, and negate the boolean.

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_logical}}
```

### Unary Operators

The next category of expressions are the _unary operators_. The only unary operator is the `IsNull` operator, which is evaluated by evaluating the operator's _comparison target_, and then comparing the result to `null`:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_unary_operators}}
```

To evaluate the comparison target, we delegate to the `eval_comparison_target` function, which pattern matches:

- A column is evaluated using the `eval_column_field_path` function.
- An aggregate is evaluated using `eval_path` (which we will talk more about when we get to [relationships](./relationships.md)) and `eval_aggregate` (which we will talk about when we get to [aggregates](./aggregates.md)).

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_comparison_target}}
```

### Binary Operators

The next category of expressions are the _binary operators_. Binary operators can be _standard_ or _custom_.

Binary operators are evaluated by evaluating their _comparison target_ and _comparison value_, and comparing them using a specific _comparison operator_:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_binary_operators}}
```

The standard binary comparison operators are:

- The equality operator, `equal`,
- The set membership operator, `in`,
- Comparison operators `less_than`, `less_than_or_equal`, `greater_than`, and `greater_than_or_equal`,
- String comparisons `contains`, `icontains`, `starts_with`, `istarts_with`, `ends_with`, `iends_with` and `like`.

`equal` is evaluated by evaluating its _comparison target_ and _comparison value_, and comparing them for equality:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_operator_eq}}
```

The ordering comparisons (`less_than`, `less_than_or_equal`, `greater_than`, and `greater_than_or_equal`) depend on their type, so first we need to determine the type of the comparison target and dispatch on it to `eval_partial_ord_comparison` to perform the actual comparisons:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_operator_ordering}}
```

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_partial_ord_comparison}}
```

The `in` operator is evaluated by evaluating its comparison target, and all of its comparison values, and testing whether the evaluated target appears in the list of evaluated values:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_binary_array_operators}}
```

String comparison operators are evaluated similarly:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_operator_string_comparisons}}
```

The reference implementation provides a single custom binary operator as an example, which is the `like` operator on strings:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_custom_binary_operators}}
```

### Scalar Array Comparison Operators

The next category of expressions are the _scalar array comparison operators_. First we must evaluate the _comparison target_ and then we can evaluate the array comparison itself.

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_array_comparison}}
```

Evaluating the array comparison is done using `eval_array_comparison`. In it, we can evaluate the two standard operators we have: `contains` and `is_empty`.

`contains` simply evaluates the comparison value and then tests whether the array from the comparison target contains any of the comparison values.

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_array_comparison_contains}}
```

`is_empty` simply checks is the comparison target array is empty:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_array_comparison_is_empty}}
```

### `EXISTS` expressions

An `EXISTS` expression is evaluated by recursively evaluating a `Query` on another source of rows, and testing to see whether the resulting `RowSet` contains any rows.

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_exists}}
```

Note in particular, we push the current row onto the stack of `scopes` before executing the inner query, so that [references to columns in those scopes](../../../specification/queries/filtering.md#referencing-a-column-from-a-collection-in-scope) can be resolved correctly.

The source of the rows is defined by `in_collection`, which we evaluate with `eval_in_collection` in order to get the rows to evaluate the inner query against. There are four different sources of rows.

#### `ExistsInCollection::Related`

The first source of rows is a related collection. We first find the specified relationship, and then use `eval_path_element` to get the rows across that relationship from the current row:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_in_collection_related}}
```

#### `ExistsInCollection::Unrelated`

The second source of rows is an unrelated collection. This simply returns all rows in that collection by using `get_collection_by_name`:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_in_collection_unrelated}}
```

#### `ExistsInCollection::NestedCollection`

The third source of rows is a nested collection. This allows us to source our rows from a nested array of objects in a column on the current row. We do this using `eval_column_field_path`.

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_in_collection_nested_collection}}
```

#### `ExistsInCollection::NestedScalarCollection`

The fourth source of rows is a nested scalar collection. This allows us to read a nested array of scalars from a column on the current row (using `eval_column_field_path`) and create a virtual row for each element in the array, placing the array element into a `__value` field on the row:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_in_collection_nested_scalar_collection}}
```
