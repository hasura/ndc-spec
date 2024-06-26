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
| `capabilities.mutation.explain` | Whether the data connector is capable of describing mutation plans |
| `capabilities.mutation.transactional` | Whether the data connector is capable of executing multiple mutations in a transaction |
| `capabilities.query.aggregates` | Whether the data connector supports [aggregate queries](queries/aggregates.md) |
| `capabilities.query.exists.named_scoped` | Whether the data connector supports [named scopes](queries/filtering.md#referencing-a-column-from-a-collection-in-scope) in exists expressions |
| `capabilities.query.exists.unrelated` | Whether the data connector supports [exists expressions](queries/filtering.md#exists-expressions) against unrelated collections |
| `capabilities.query.explain` | Whether the data connector is capable of describing query plans |
| `capabilities.query.nested_fields.filter_by` | Whether the data connector is capable of filtering by nested fields |
| `capabilities.query.nested_fields.order_by` | Whether the data connector is capable of ordering by nested fields |
| `capabilities.query.variables` | Whether the data connector supports [queries with variables](queries/variables.md) |
| `capabilities.relationships.order_by_aggregate` | Whether order by clauses can include aggregates |
| `capabilities.relationships.relation_comparisons` | Whether comparisons can include columns reachable via [relationships](queries/relationships.md) |
| `capabilities.relationships` | Whether the data connector supports [relationships](queries/relationships.md) |
| `version` | A [semantic version number](https://semver.org) of this specification which the data connector claims to implement |

## See also

- Type [`Capabilities`](../reference/types.md#capabilities)
- Type [`CapabilitiesResponse`](../reference/types.md#capabilitiesresponse)
- Type [`QueryCapabilities`](../reference/types.md#querycapabilities)
- Type [`NestedFieldCapabilities`](../reference/types.md#nestedfieldcapabilities)
- Type [`MutationCapabilities`](../reference/types.md#mutationcapabilities)
- Type [`RelationshipCapabilities`](../reference/types.md#relationshipcapabilities)
