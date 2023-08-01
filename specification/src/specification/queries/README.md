# Queries

The query endpoint accepts a query request, containing expressions to be evaluated in the context the data source, and returns a response consisting of relevant rows of data.

The structure and requirements for specific fields listed below will be covered in subsequent chapters.

## Request

```
POST /query
```

## Request

See [`QueryRequest`](../../reference/types.md#queryrequest)

## Request Fields

| Name | Description |
|------|-------------|
| `collection` | The name of a collection to query |
| `query` | The query syntax tree |
| `arguments` | Values to be provided to any top-level [collection arguments](./arguments.md) |
| `collection_relationships` | Any [relationships](./relationships.md) between collections involved in the query request |
| `variables` | One set of [named variables](./variables.md) for each rowset to fetch. Each variable set should be subtituted in turn, and a fresh set of rows returned. |

## Response

See [`QueryResponse`](../../reference/types.md#queryresponse)

## Requirements

- If the request specifies `variables`, then the response must contain one [`RowSet`](../../reference/types.md#rowset) for each collection of variables provided. If not, the data connector should respond as if `variables` were set to a single empty collection of variables: `[{}]`.
- If the request specifies `fields`, then the response must contain `rows` according to the [schema](../schema/README.md) advertised for the requested `collection`.
- If the request specifies `aggregates` then the response must contain `aggregates`, with one response key per requested aggregate, using the same keys. See [aggregates](./aggregates.md).
- If the request specifies `arguments`, then the implementation must validate the provided arguments against the types specified by the collection's [schema](../schema/README.md). See [arguments](./arguments.md).