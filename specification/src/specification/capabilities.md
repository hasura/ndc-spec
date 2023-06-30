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
{
  "versions": "^1.0.0",
  "capabilities": {
    "query": {
        "foreach": {},
        "relation_comparisons": {},
        "order_by_aggregate": {}
    },
    "mutations": {
        "nested_inserts": {},
        "returning": {}
    },
    "explain": {},
    "relationships": {},
  }
}
```

## Response Fields

_TODO_: nested_inserts and relation_comparisons seem like special cases of relationships

_TODO_: the code doesn't actually follow this response format right now

| Name | Description |
|------|-------------|
| `versions` | A [semantic versioning](https://semver.org) range of API versions which the data connector claims to implement |
| `capabilities.query.foreach` | Whether the data connector supports [`foreach` queries](queries/foreach.md) |
| `capabilities.query.relation_comparisons` | Whether comparisons can include columns reachable via [relationships](queries/relationships.md) |
| `capabilities.query.order_by_aggregate` | Whether order by clauses can include aggregates |
| `capabilities.mutations.nested_inserts` | Whether nested insert mutations are supported |
| `capabilities.mutations.returning` | Whether mutations return rows of modified data |
| `capabilities.explain` | Whether the data connector is capable of describing query plans |
| `capabilities.relationships` | Whether the data connector supports [relationships](queries/relationships.md) |

## See also

- Type [`Capabilities`](../reference/types.md#capabilities)
- Type [`CapabilitiesResponse`](../reference/types.md#capabilitiesresponse)
- Type [`MutationCapabilities`](../reference/types.md#mutationcapabilities)
- Type [`QueryCapabilities`](../reference/types.md#querycapabilities)