# Capabilities

The capabilities endpoint provides metadata about the features which the data connector (and data source) support.

## Request

```
GET /capabilities
```

## Response

See [`CapabilitiesResponse`](../reference/types.md#capabilitiesresponse)

### Example

```json
{{#include ../../../ndc-reference/tests/capabilities/expected.json}}
```

## Response Fields

| Name | Description |
|------|-------------|
| `versions` | A [semantic versioning](https://semver.org) range of API versions which the data connector
| `capabilities.explain` | Whether the data connector is capable of describing query plans |claims to implement |
| `capabilities.mutation` | Whether the data connector is capable of executing mutations |claims to implement |
| `capabilities.mutation.transactional` | Whether the data connector is capable of executing multiple mutations in a transaction |claims to implement |
| `capabilities.query.aggregates` | Whether the data connector supports [aggregate queries](queries/aggregates.md) |
| `capabilities.query.variables` | Whether the data connector supports [queries with variables](queries/variables.md) |
| `capabilities.relationships` | Whether the data connector supports [relationships](queries/relationships.md) |
| `capabilities.relationships.order_by_aggregate` | Whether order by clauses can include aggregates |
| `capabilities.relationships.relation_comparisons` | Whether comparisons can include columns reachable via [relationships](queries/relationships.md) |

## See also

- Type [`Capabilities`](../reference/types.md#capabilities)
- Type [`CapabilitiesResponse`](../reference/types.md#capabilitiesresponse)
- Type [`QueryCapabilities`](../reference/types.md#querycapabilities)
- Type [`MutationCapabilities`](../reference/types.md#mutationcapabilities)
