# Newtypes in `ndc-models`

## Purpose

Most string types in `ndc_models` are raw `String`s. Let's use newtypes to distinguish bewteen different syntactic categories of strings.

## Proposal

We'll start the following list, and then expand later if necessary:

- `ColumnName`
- `RelationshipName`
- `ArgumentName`
- `CollectionName`
- `ProcedureName`
- `ScalarTypeName`
- `ObjectTypeName`
- `FunctionName`
- `ComparisonOperatorName`

Since they are all immutable, we will implement them with `SmolStr` underneath, and ensure they have a `Display` method to allow connector writers to turn them into `String` should they need to.
