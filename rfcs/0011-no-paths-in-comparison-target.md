# Remove `path` from `ComparisonTarget`

## Purpose

There is duplication in the current AST:

```rust
pub enum ComparisonTarget {
    Column {
        name: String,
        path: Vec<PathElement>
        ...
    },
    ...
}
```

A `ComparisonTarget`, which appears as both the left-hand side, and possibly also the right-hand side of a comparison (via `ComparisonValue::Column`) contains a `path` property, which allows the caller to pick out a column of a related table. The `path` can traverse multiple object and array relationships.

An array relationship is implicitly existentially quantified. So for example, we can query for artist rows where `albums.title = "foo"`, and the meaning is that _there exists_ a related album row _such that_ its title equals `"foo"`.

On the right hand side, the same rule applies, so we could also say something like `name = albums.title` (using a column comparison), meaning _there exists_ a related album row whose title is the same as the artist's name.

The first of these cases overlaps partially with the functionality provided by `Expression::Exists` (called "`EXISTS` predicates" in the spec), because we can use `Exists` to change context to the related table, and evaluate the comparison there. Our `albums.title = "foo"` example becomes `∃ album ∈ albums. album.title = "foo"`.

However, it's not exactly duplicated functionality, because `EXISTS` changes the context in which a `ComparisonValue` is evaluated, from the original table to the related table. Therefore, if the right-hand side is a `ComparisonValue::Column`, then there may be no way to express this using `EXISTS`. For example, the predicate `foo.bar.baz.{column} = quux.{column}` cannot be expressed using only `EXISTS` when `foo` and `baz` are array relationships and `bar` is an object relationship, because `quux` is not in scope in the new context (`bar`) (and the objct relationship `bar` cannot be "inverted" in order to bring `foo` back into scope like we might be able to do in some simpler cases).

With two possible places in the IR for implicit existential quantification, it becomes hard to describe the semantics clearly. This can be seen by looking at the code for the reference implementation, where we need to have several functions return types wrapped in `Vec` to indicate a set of possible alternatives.

## Proposal

The proposal is to remove `path` from `ComparisonTarget`, and to add it instead to `ComparisonValue::Column`. This means that we can express implicit existential quantifiers on the right-hand side of a comparison operation, but not on the left. On the left-hand side, the recommendation is to use `EXISTS`.

This means that we can translate most queries, but not all:

- The `artists` query `albums.title = "foo"` becomes `∃ album ∈ albums. album.title = "foo"`.
- The `albums` query `title = artist.name` can still be expressed since all quantifiation occurs on the right-hand side.
- The `albums` query `tracks.title = artist.name` cannot naively be translated, but an equivalent query can be written: `∃ track ∈ tracks. track.title = track.album.artist.name`.
- Queries which involve a combination of object and array relationships on the left hand side of a comparison, and a column on the right, may not be expressible using the new IR.

### Future work: named scopes

If we feel it is necessary to support queries with column comparisons in `EXISTS` scope, then we can support that with a capability.

If a connector supports "named scopes" then we can send IR which refers to columns in an outer scope. We can do this by adding a new optional `scope: Option<u8>` field to `ComparisonValue::Column`. The integer-valued scope refers to one of the enclosing scopes (outside an enclosing `EXISTS`): `1` refers to the immediately-enclosing scope, `2` refers to the next enclosing scope, and so on.

For example `tracks.title = artist.name` becomes `∃ track ∈ tracks. track.title = {1}.artist.name` where `{1}` refers to the immediately enclosing `albums` scope.

The earlier `foo.bar.baz.{column} = quux.{column}` example becomes

```
∃ foo ∈ foo.
  ∃ bar ∈ bar.
    ∃ baz ∈ baz.
      baz.{column} = {3}.quux.{column}
```

where `{3}` is used to escape 3 levels of existential quantification.

## Alternatives

### Disallow array relationships in `path`

A simpler change is to only allow object relationships in `path`s. This is already the case for `path` in `OrderByTarget::Column`.

We can disallow array relationships on the left-hand and right-hand sides independently, and we can also use capabilities to turn these features back on.

## Open Questions

- Are there permissions use cases which we would lose, which we consider essential?
- Are "named scopes" worth it? Necessary?
