# Boolean Expression Types

## Purpose

In order to implement useful permissions, it is desirable to be able to provide _predicates_ as input arguments to functions and procedures. Predicate types for expressions are already defined by the spec, so the present proposal is simply to reify them as new objects in the type language.

## Proposal

- The `Type` enum will be extended with a new constructor `Predicate { object_type_name: String }`, which will refer to the predicate type for the named object type. The value-level representation of these predicates is already defined by the spec (for collection types, at least), and can be reused.
- An input argument with a predicate type can use the existing client types to parse its input value.
- This extends the usefulness of functions and procedures, but even a function supporting predicate arguments is still more general than a full collection, because a collection requires ordering and pagination, which a source may not support. Thus, functions can now implement a wider variety of sources in a way which supports a useful notion of permissions.

## Changes Required

- Add the new constructor to `Type`.

## Questions

- Predicates are useful as input arguments, but the type language also extends the types of _fields_ and _return types_. Is this useful?
  - Maybe, maybe not. But there is no harm in allowing the user to express this. We would most likely not implement field selection and filtering on these new types, so they would be abstract from an output point of view. But it's not inconceivable that a connector might want to return a predicate for some reason.