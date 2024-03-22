# Mutations

The mutation endpoint accepts a mutation request, containing a collection of mutation operations to be performed transactionally in the context the data source, and returns a response containing a result for each operation.

The structure and requirements for specific fields listed below will be covered in subsequent chapters.

## Request

```
POST /mutation
```

## Request

See [`MutationRequest`](../../reference/types.md#mutationrequest)

## Request Fields

| Name | Description |
|------|-------------|
| `operations` | A list of mutation operations to perform |
| `collection_relationships` | Any [relationships](../queries/relationships.md) between collections involved in the mutation request |

## Mutation Operations

Each operation is described by a [`MutationOperation`](../../reference/types.md#mutationoperation) structure, which can be one of several types. However, currently [procedures](./procedures.md) are the only supported operation type.

### Multiple Operations

If the `mutation.transactional` capability is enabled, then the caller may provide multiple operations in a single request.
Otherwise, the caller must provide exactly one operation.

The intent is that multiple operations ought to be performed together in a single transaction.
That is, they should all succeed, or all fail together. If any operation fails, then a single `ErrorResponse` should capture
the failure, and none of the operations should effect any changes to the data source.

## Response

See [`MutationResponse`](../../reference/types.md#mutationresponse)

## Requirements

- The `operation_results` field of the [`MutationResponse`](../../reference/types.md#mutationresponse) should contain one [`MutationOperationResults`](../../reference/types.md#mutationoperationresults) structure for each requested operation in the [`MutationRequest`](../../reference/types.md#mutationrequest).