# Named Scopes

## Purpose

There are currently two scopes that a `ComparisonTarget` can refer to: the current collection, and the "root" collection. 

The root collection is the collection in scope at the nearest enclosing `Query`. That is, it changes at a `Field::Relationship` boundary. Root collection references are useful inside `Expression::Exists`, where the "current row" differs from the root collection row.

However, when we have nested `EXISTS` expressions, there are more than two scopes in play, and it can be useful to refer to columns in intermediate scopes.

### Example - `EXISTS` without a relationship

`ExistsInCollection::Unrelated` is of limited utility right now, because the caller can only refer to the root collection, and not the collection in scope outside the `EXISTS`.

### Example - complex column comparisons

The recent [`ComparisonTarget` RFC](./0011-no-paths-in-comparison-target.md) gives an example. A predicate like `foo.bar.baz.{column} = quux.{column}` should be translated to a nested `EXISTS` expression:

```
∃ foo ∈ foo. 
  ∃ bar ∈ bar.
    ∃ baz ∈ baz.
      baz.{column} = {3}.quux.{column}
```

If `quux` is a field from the outer scope, then inside the innermost exists, it exists on the scope denoted as `{3}` here. That is, the scope in position 3 on the stack. See the linked RFC for more details.

## Proposal

The proposal is to extend `ComparisonValue` (not `ComparisonTarget`) to allow the caller to refer to these intermediate scopes:

```rust
pub enum ComparisonValue {
    Column {
        name: String,
        path: Vec<PathElement>,
        ...
        scope: Option<usize>,
    },
    ...
}
```

Here, `scope` is an optional integer pointing to a position on a stack of scopes. As a connector enters a `Field::Relationship`, it should reset the stack to empty. As it enters an `Expression::Exists`, it should push the current row onto the stack.

So scope number zero refers to the current row in the current collection; scope number one refers to the row which was in scope in the immediately enclosing `EXISTS` expression; scope number two refers to the the row in scope in the next enclosing `EXISTS` expression, and so on. A query is only well-formed if these indices never meet or exceed the maximum number of enclosing `EXISTS` expressions up to the nearest enclosing `Query`.

## Notes

- Why does the scopes stack get reset at relationship boundaries?
  - Because a `Query` should have the same meaning regardless of where it appears in a `QueryRequest`.

