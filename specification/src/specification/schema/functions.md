# Functions

Functions are a special case of [collections](./collections.md), which are identified separately in the schema for convenience. 

A function is a collection which returns a single row and a single column, named `__value`. Like collections, functions can have arguments. Unlike collections, functions cannot be used by the mutations endpoint, do not describe constraints, and only provide a type for the `__value` column, not the name of an object type.

_Note_: even though a function acts like a collection returning a row type with a single column, there is no need to define and name such a type in the `object_types` section of the schema response.

To describe a function, add a [`FunctionInfo`](../../reference/types.md#FunctionInfo) structure to the `functions` field of the schema response.

## Example

```json
{
  "functions": [
    {
      "name": "latest_article_id",
      "description": "Get the ID of the most recent article",
      "arguments": {},
      "result_type": {
        "type": "nullable",
        "underlying_type": {
          "type": "named",
          "name": "Int"
        }
      }
    }
  ],
  ...
}
```


## See also

- Type [`FunctionInfo`](../../reference/types.md#FunctionInfo)