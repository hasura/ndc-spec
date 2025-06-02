# Request-level arguments

Request-level arguments are arguments that are passed to the request itself, rather than to a specific query or procedure.

These can be used to pass things like connection strings or authentication tokens that change dynamically.

## Example

```json
{
  "query_arguments": {
    "connection_string": {
      "description": "Connection string for data source",
      "type": {
        "type": "named",
        "name": "text"
      }
    },
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
