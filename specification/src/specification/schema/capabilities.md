# Capabilities

The schema response should also provide any capability-specific data, based on the set of enabled [capabilities](../capabilities.md).

## Requirements

- If the `query.aggregates.filter_by` capability is enabled, then the schema response should include the `capabilities.query.aggregates.filter_by` object, which has type [`AggregateFilterByCapabilitiesSchemaInfo`](../../reference/types.md#aggregatefilterbycapabilitiesschemainfo).
  - This object should indicate those scalar types used as aggregate result types, in order to implement [filtering by aggregates](../queries/filtering.md#computing-an-aggregate).