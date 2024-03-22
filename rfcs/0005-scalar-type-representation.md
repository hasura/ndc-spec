# Scalar Type Representations

## Purpose

Several connectors have a useful concept of enumerable input and output types. For example, Postgres and TypeScript both have enum types. On the client side, we can generate GraphQL enum types if we can provide a hint to the client about the representation of such values.

Right now, we don't have any notion of type representation for scalar types, so we can solve the more general problem at the same time, by adding optional _type representation hints_ to scalar types in the NDC schema.

## Proposal

- Add a `representation` field to `ScalarType`, which is optional with type `TypeRepresentation`:

    ```rust
    pub enum TypeRepresentation {
        String,
        Float,
        Boolean,
        Enum { one_of: Vec<String> },
    }
    ```
- In `ndc-test`:
  - We don't need to perform any particular validation for these values, except to maybe make sure that `one_of` doesn't include duplicate values.
  - We can now synthesize values of some types (boolean, enum) without examples, including function arguments.
  - If a `representation` is provided, then we can perform additional response validation in each test case.

## Alternative Designs

### Extend `Type` to add enums

An alternative is to add enums to the type language, in the form of a new constructor for `Type`.

However, we want enum types to be scalar types, because we want to attach comparison and aggregation operators to enums.