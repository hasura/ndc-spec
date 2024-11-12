# Capabilities

The schema response should also provide any capability-specific data, based on the set of enabled [capabilities](../capabilities.md).

## Requirements

- If the `query.aggregates` capability is enabled, then the schema response should include the `capabilities.query.aggregates` object, which has type [`AggregateCapabilitiesSchemaInfo`](../../reference/types.md#aggregatecapabilitiesschemainfo).
  - This object should indicate the scalar type used as count aggregate result type, in order to implement [aggregates](../queries/aggregates.md).

## Example

```json
{
  ...
  "capabilities": {
    "query": {
      "aggregates": {
        "count_scalar_type": "Int"
      }
    }
  }
}
```
