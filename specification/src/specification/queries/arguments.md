# Arguments

_Collection arguments_ parameterize an entire collection, and must be provided in queries wherever the collection is referenced, either directly, or via relationships. 

_Field_ arguments parameterize a single field, and must be provided wherever that field is referenced.

## Collection Arguments

Collection arguments should be provided in the `QueryRequest` anywhere a collection is referenced. The set of provided arguments should be compatible with the list of arguments required by the corresponding [collection in the schema response](../schema/collections.md).

### Specifying arguments to the top-level collection

Collection arguments should be provided as key-value pairs in the `arguments` property of the top-level `QueryRequest` object:

```json
{{#include ../../../../ndc-reference/tests/query/table_argument/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/table_argument/request.json:3: }}
```

### Relationships

[Relationships](./relationships.md) can specify values for arguments on their target collection:

```json
{{#include ../../../../ndc-reference/tests/query/table_argument_relationship_1/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/table_argument_relationship_1/request.json:3: }}
```

Any arguments which are not defined by the relationship itself should be specified where the relationship is used. For example, here the `author_id` argument can be moved from the relationship definition to the field which uses it:

```json
{{#include ../../../../ndc-reference/tests/query/table_argument_relationship_2/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/table_argument_relationship_2/request.json:3: }}
```

### Collection arguments in predicates

Arguments must be specified in predicates whenever a reference to a secondary collection is required.

For example, in an `EXISTS` expression, if the target collection has arguments:

```json
{{#include ../../../../ndc-reference/tests/query/table_argument_exists/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/table_argument_exists/request.json:3: }}
```

Or when a predicate expression matches a column from a related collection:

```json
{{#include ../../../../ndc-reference/tests/query/table_argument_predicate/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/table_argument_predicate/request.json:3: }}
```

### Collection arguments in `order_by`

Arguments must be specified when an `OrderByElement` references a related collection.

For example, when ordering by an aggregate of rows in a related collection, and that collection has arguments:

```json
{{#include ../../../../ndc-reference/tests/query/table_argument_order_by/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/table_argument_order_by/request.json:3: }}
```

## Field Arguments

Field arguments can be provided to any field requested (in addition to those described for top-level collections).
These are specified in the [schema response](../schema/object-types.md) and their use is described in [field selection](./field-selection.md). Their specification and usage matches that of collection arguments above.