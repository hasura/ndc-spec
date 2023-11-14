# Schema

The schema endpoint defines any types used by the data connector, and describes the collections and their columns, functions, and any procedures.

The schema endpoint is used to specify the behavior of a data connector, so that it can be tested, verified, and used by tools such as code generators. It is primarily provided by data connector implementors as a development and specification tool, and it is not expected to be used at "runtime", in the same sense that the `/query` and `/mutation` endpoints would be.

## Request

```
GET /schema
```

## Response

See [`SchemaResponse`](../../reference/types.md#schemaresponse)

### Example

```json
{{#include ../../../../ndc-reference/tests/schema/expected.json}}
```

## Response Fields

| Name | Description |
|------|-------------|
| `scalar_types` | [Scalar Types](scalar-types.md) |
| `object_types` | [Object Types](object-types.md) |
| `collections` | [Collection](collections.md) |
| `functions` | [Functions](functions.md) |
| `procedures` | [Procedures](procedures.md) |
