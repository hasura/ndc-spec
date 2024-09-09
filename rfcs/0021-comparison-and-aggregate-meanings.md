# Comparison and Aggregate Meanings

## Purpose

We already have a way to identify certain comparison operators as special - `equal` and `in` are special cases in comparison operator definitions. This allows engine to identify which operators can be used for expressing equalities in e.g. primary key lookups.

However, there are more standardizable comparison operators, and we can introduce a similar notion for aggregate operators.

New abstractions are only useful if they have abstract clients, so we need to find some candidates. GraphQL won't use all comparison and aggregate operators abstractly, but the new SQL layer will. When the SQL user writes `column > value`, we need to know which NDC comparion operator to translate this to. Similarly, when the user writes `SUM(column)`, we need a way to translate this predictably to an aggregate function with the right semantics.

## Proposal

### New `ComparisonOperatorDefinition` types

Add the following variants to `ComparisonOperatorDefinition`:

```rust
 pub enum ComparisonOperatorDefinition {
     // ...
     LessThan,
     LessThanOrEqual,
     GreaterThan,
     GreaterThanOrEqual,
     // ...
 }
```

Each of these needs to be given well-defined semantics in the specification.

There are two ways we can go about extracting a specification here: either based on the likely capabilities of our various connectors, or based on the needs of our clients.

If we go the first route, we're likely to run into a variety of unusual implementations of these operators, e.g. databases will depend on collation settings, some operators may not be intuitive, e.g. operators may not be transitive, may use 3-valued logic, etc.

For certain type representations, we could use existing specs, e.g. IEEE 754 defines these operators for floating point numbers. However, for other types, it will depend on the database in question, e.g. collation settings will affect these operators for strings. 

If we start with the needs of the clients then what does SQL/datafusion require of these operators?

- A quick look through the datafusion code shows that it rewrites `NOT (a < b)` into `a >= b`, and `a < b` into `b > a` in some cases for canonicalization. We should require these rules on the NDC side.
- datafusion also implements an interval arithmetic and constraints library, so we should take a look to see what that implies for these operators too.
- In general, our constraints may change in future if new rewrites get added, so we would need to be careful there.

### Add meanings for aggregate functions

Change `AggregateFunctionDefinition` into an `enum`:

```rust
pub enum AggregateFunctionDefinition {
    Sum,
    Min, 
    Max,
    Average,
    Custom {
        result_type: Type,
    },
}
```

Again, these will need to be standardized. 

We can require `Sum` and `Average` to be only defined on types with numeric type representations, and then simply define them in terms of IEEE 754 floats or integers.

`Min` and `Max` ought to also be definable on strings and date/times. We could require these to be compatible with `<` and `<=`, or we could rely on standard definitions from e.g. ISO 8601.

_TODO_: consider datafusion rewrites.

## Future Work

Other possible comparison operators (from https://datafusion.apache.org/user-guide/sql/operators.html#other-operators):

- `IS DISTINCT FROM`, `IS NOT DISTINCT FROM`
- `~`, `~*`, `!~`, `!~*` (regex-based matches, may be difficult to standardize across implementions, might consider "starts with", "contains" and "ends with" instead)
- `@>`, `<@` (array operators)

Other possible aggregate functions (from https://datafusion.apache.org/user-guide/sql/operators.html#comparison-operators):

- `bit_and`, `bit_or`, `bit_xor`
- `bool_and`, `bool_or`
- `mean`, `median`
- `array_agg`
- `first_value`, `last_value`