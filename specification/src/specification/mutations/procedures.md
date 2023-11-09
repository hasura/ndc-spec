# Procedures

A procedure which is [described in the schema](../schema/procedures.md) can be invoked using a [`MutationOperation`](../../reference/types.md#mutationoperation).

The operation should specify the procedure name, any arguments, and a list of [`Field`](../../reference/types.md#field)s to be returned.

## Requirements

- The `affected_rows` field in the corresponding [`MutationOperationResults`](../../reference/types.md#mutationoperationresults) structure should indicate the number of rows in the data source which were modified as a result of the operation.
- The `returning` field in the corresponding [`MutationOperationResults`](../../reference/types.md#mutationoperationresults) structure should contain a single row, with a single column named `__value`. That column should contain the result of the operation, which has a value compatible with the return type of the procedure.