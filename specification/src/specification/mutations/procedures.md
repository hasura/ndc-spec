# Procedures

A procedure which is [described in the schema](../schema/procedures.md) can be invoked using a [`MutationOperation`](../../reference/types.md#mutationoperation).

The operation should specify the procedure name, any arguments, and a list of [`Field`](../../reference/types.md#field)s to be returned.

_Note_: just as for [functions](../queries/functions.md), fields to return can include [relationships](../queries/relationships.md) or [nested fields](../queries/field-selection.md#nested-fields). However, unlike functions, procedures do not need to wrap their result in a `__value` field, so top-level fields can be extracted without use of nested field queries.

## Requirements

- The [`MutationResponse`](../../reference/types.md#mutationresponse) structure will contain a [`MutationOperationResults`](../../reference/types.md#mutationoperationresults) structure for the procedure response. This structure should have type `procedure` and contain a `result` field with a result of the type indicated in the [schema response](../schema/procedures.md).