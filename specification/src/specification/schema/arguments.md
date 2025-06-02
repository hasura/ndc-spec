# Request-level arguments

Request-level arguments are arguments that are passed to the request itself, rather than to a specific query or procedure.

These can be used to pass things like authentication tokens that change dynamically.

## Example

```json
{
  "query_arguments": {
    "connection_timeout": {
      "description": "Timeout for connecting to data source (ms)",
      "type": {
        "type": "named",
        "name": "int"
      }
    }
  },
  "mutation_arguments": {
    "use_transaction": {
      "description": "Whether to run the mutations in this request within a single transaction",
      "type": {
        "type": "named",
        "name": "bool"
      }
    }
  }
  ...
}
```
