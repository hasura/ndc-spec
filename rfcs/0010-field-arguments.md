# Field Arguments

The ability to provide arguments to fields broadens the use-cases that NDC can support.

This can be seen as generalising `fields` to `functions` or, simply as adding context to fields.

## Proposal

This proposes an update to the NDC-Spec to allow arguments to fields.

The motivation for this change is to generalise the API surface to allow more expressive queries - This should be very similar to collection arguments in practice but can apply to any field (especially nested), not just top-level collections.

Some examples of why this might be useful:

* JSON Operations: For a JSON field in a PG table, several JSON functions could be exposed as fields but they require arguments to be useful. Such as [#>](https://www.postgresql.org/docs/9.3/functions-json.html)
* Vector Operations for LLMs etc.
* Advanced Geo Operations
* GraphQL federation: Forwarding Hasura GQL schemas over NDC will require this change if we want to expose root fields as commands instead of collections.

The schema is extended with:

```rust
pub struct ObjectField {
  ...
  pub arguments: BTreeMap<String, ArgumentInfo>,
}
```

and queries are extended with:

```rust
pub enum Field {
  Column {
    ...
    arguments: BTreeMap<String, Argument>,
  }
  ...
}
```

This mirrors the existing implementation for collection arguments.

## Implications

NDC schema and query invocation:

* When the schema indicates that a field has arguments then they must be provided in a query.
* Optional arguments must be explicitly supplied with `Argument::Literal { Value::Null }`.
* If all arguments are nullable then the field may be referenced without arguments or parenteses
  for backwards compatibility purposes, however arguments should be applied going forward.

Engine interactions:

* Query field arguments will need to be translated to NDC field arguments
* NDC schema responses will need to be translated into graphql schemas
* Variables need to be bound to field arguments if they are not supplied as scalars
