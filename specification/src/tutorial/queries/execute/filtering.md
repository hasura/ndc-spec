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

- A column is evaluated using the `eval_path` function, which we will cover when we talk about [relationships](./relationships.md).
- A _root collection_ column (that is, a column from the _root collection_, or collection used by the nearest enclosing [`Query`](../../../reference/types.md#query)) is evaluated using `eval_column`. You may have noticed the additional argument, `root`, which has been passed down through every function call so far - this is to track the root collection for exactly this case.

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_comparison_target}}
```

### Binary Operators

The next category of expressions are the _binary operators_. Binary operators can be _standard_ or _custom_.

The only standard binary operators are the `equal` and `in` operators. 

`equal` evaluated by evaluating its _comparison target_ and _comparison value_, and comparing them for equality:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_binary_operators}}
```

The `in` operator is evaluated by evaluating its comparison target, and all of its comparison values, and testing whether the evaluated target appears in the list of evaluated values:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_binary_array_operators}}
```

The reference implementation provides a single custom binary operator as an example, which is the `like` operator on strings:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_custom_binary_operators}}
```

### `EXISTS` expressions

An `EXISTS` expression is evaluated by recursively evaluating a `Query` on a related collection, and testing to see whether the resulting `RowSet` contains any rows:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:eval_expression_exists}}
```