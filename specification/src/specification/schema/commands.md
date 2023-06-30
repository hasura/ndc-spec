# Commands

The schema should define metadata for each _command_ which the data connector implements.

Each command is defined by its name, any arguments types and a result type.

To describe a command, add a [`CommandInfo`](../../reference/types.md#commandinfo) structure to the `command` field of the schema response.

## Example

```json
{
  "commands": [
    {
      "name": "upsert_article",
      "description": "Insert or update an article",
      "arguments": {
        "article": {
          "description": "The article to insert or update",
          "type": {
            "type": "named",
            "name": "article"
          }
        }
      },
      "result_type": {
        "type": "named",
        "name": "article"
      }
    }
  ],
  ...
}
```

## See also

- Type [`CommandInfo`](../../reference/types.md#commandinfo)