# Procedures

The schema should define metadata for each _procedure_ which the data connector implements.

Each procedure is defined by its name, any arguments types and a result type.

To describe a procedure, add a [`ProcedureInfo`](../../reference/types.md#procedureinfo) structure to the `procedure` field of the schema response.

## Example

```json
{
  "procedures": [
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

- Type [`ProcedureInfo`](../../reference/types.md#procedureinfo)