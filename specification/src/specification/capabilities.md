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
{{#include ../../../ndc-reference/tests/capabilities/expected.snap:5:}}
```

## Response Fields

| Name           | Description                                                                                                        |
| -------------- | ------------------------------------------------------------------------------------------------------------------ |
| `version`      | A [semantic version number](https://semver.org) of this specification which the data connector claims to implement |
| `capabilities` | The capabilities that this connector supports, see [below](#capabilities-fields)                                   |

### Capabilities Fields

These fields are set underneath the `capabilities` property on the `CapabilitiesResponse` object:

| Name                                                   | Description                                                                                                                                                                                 |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `mutation.explain`                                     | Whether the data connector is capable of [describing mutation plans](explain.md)                                                                                                            |
| `mutation.transactional`                               | Whether the data connector is capable of executing [multiple mutations in a transaction](mutations/README.md#multiple-operations)                                                           |
| `query.aggregates`                                     | Whether the data connector supports [aggregate queries](queries/aggregates.md). The [schema `capabilities.query.aggregates`](schema/capabilities.md) should also be returned.               |
| `query.aggregates.filter_by`                           | Whether the data connector supports [filtering by aggregated values](queries/filtering.md#computing-an-aggregate)                                                                           |
| `query.aggregates.group_by`                            | Whether the data connector supports [grouping operations](queries/grouping.md)                                                                                                              |
| `query.aggregates.group_by.filter`                     | Whether the data connector supports [filtering on groups](queries/grouping.md#filtering)                                                                                                    |
| `query.aggregates.group_by.order`                      | Whether the data connector supports [ordering on groups](queries/grouping.md#ordering)                                                                                                      |
| `query.aggregates.group_by.paginate`                   | Whether the data connector supports [pagination on groups](queries/grouping.md#pagination)                                                                                                  |
| `query.exists.named_scopes`                            | Whether the data connector supports [named scopes](queries/filtering.md#referencing-a-column-from-a-collection-in-scope) in exists expressions                                              |
| `query.exists.nested_collections`                      | Whether the data connector supports [exists expressions](queries/filtering.md#exists-expressions) against [nested collections](queries/field-selection.md#nested-collections)               |
| `query.exists.nested_scalar_collections`               | Whether the data connector supports [exists expressions](queries/filtering.md#exists-expressions) against [nested scalar collections](queries/field-selection.md#nested-scalar-collections) |
| `query.exists.unrelated`                               | Whether the data connector supports [exists expressions](queries/filtering.md#exists-expressions) against unrelated collections                                                             |
| `query.explain`                                        | Whether the data connector is capable of [describing query plans](explain.md)                                                                                                               |
| `query.nested_fields.aggregates`                       | Whether the data connector is capable of [aggregating fields in nested objects](queries/aggregates.md)                                                                                      |
| `query.nested_fields.filter_by`                        | Whether the data connector is capable of [filtering by nested fields](queries/filtering.md#referencing-nested-fields-within-columns)                                                        |
| `query.nested_fields.filter_by.nested_arrays`          | Whether the data connector is capable of filtering over nested arrays using [`array_comparison` expressions](queries/filtering.md#nested-array-comparison-operators)                        |
| `query.nested_fields.filter_by.nested_arrays.contains` | Whether the data connector is capable of filtering over nested arrays using the [contains operator](queries/filtering.md#nested-array-comparison-operators)                                 |
| `query.nested_fields.filter_by.nested_arrays.is_empty` | Whether the data connector is capable of filtering over nested arrays using the is [empty operator](queries/filtering.md#nested-array-comparison-operators)                                 |
| `query.nested_fields.nested_collections`               | Whether the data connector supports [nested collection field queries](queries/field-selection.md#nested-collections)                                                                        |
| `query.nested_fields.order_by`                         | Whether the data connector is capable of [ordering by nested fields](queries/sorting.md#type-column)                                                                                        |
| `query.variables`                                      | Whether the data connector supports [queries with variables](queries/variables.md)                                                                                                          |
| `relationships`                                        | Whether the data connector supports [relationships](queries/relationships.md)                                                                                                               |
| `relationships.nested`                                 | Whether the data connector supports relationships that can [start from or end with columns in nested objects](queries/relationships.md#column-mappings)                                     |
| `relationships.nested.array`                           | Whether the data connector supports relationships that can [start from columns inside nested objects inside nested arrays](queries/relationships.md#column-mappings)                        |
| `relationships.nested.filtering`                       | Whether the data connector supports using relationships that can [start from columns inside nested objects](queries/relationships.md#column-mappings) while filtering                       |
| `relationships.nested.ordering`                        | Whether the data connector supports using relationships that can [start from columns inside nested objects](queries/relationships.md#column-mappings) while ordering                        |
| `relationships.order_by_aggregate`                     | Whether order by clauses can [include aggregates](queries/sorting.md#type-aggregate)                                                                                                        |
| `relationships.relation_comparisons`                   | Whether comparisons between two columns can include a [value column](queries/filtering.md#values-in-binary-operators) that is across a [relationship](queries/relationships.md)             |

## See also

- Type [`Capabilities`](../reference/types.md#capabilities)
- Type [`CapabilitiesResponse`](../reference/types.md#capabilitiesresponse)
- Type [`QueryCapabilities`](../reference/types.md#querycapabilities)
- Type [`NestedFieldCapabilities`](../reference/types.md#nestedfieldcapabilities)
- Type [`MutationCapabilities`](../reference/types.md#mutationcapabilities)
- Type [`RelationshipCapabilities`](../reference/types.md#relationshipcapabilities)
