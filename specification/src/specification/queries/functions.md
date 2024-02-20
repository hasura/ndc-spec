# Functions

A [function](../schema/functions.md) is invoked in a query request in exactly the same way as any other collection - recall that a function is simply a collection which returns a single row, and a single column, named `__value`.

Because a function returns a single row, many query capabilities are limited in their usefulness:

- It would not make sense to specify `limit` or `offset`,
- Sorting has no effect
- Filtering can only remove the whole result row, based on some condition expressed in terms of the _result_.

However, some query functions are still useful in the context of functions:

- The caller can request a subset of the full result, by using [nested field queries](./field-selection.md#nested-fields),
- A function can be the source or target of a [relationship](./relationships.md),
- Function [arguments](./arguments.md) are specified in the same way as collection arguments, and can also be specified using [variables](./variables.md).

## Examples

### A function returning a scalar value

This example uses the `latest_article_id` function, which returns a scalar type:

```json
{{#include ../../../../ndc-reference/tests/query/get_max_article_id/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/get_max_article_id/request.json:3: }}
```

The response JSON includes the requested data in the special `__value` field:

```json
{{#include ../../../../ndc-reference/tests/query/get_max_article_id/expected.json }}
```

### A function returning an object type

This example uses the `latest_article` function instead, which returns the full `article` object. To query the object structure, it uses a [nested field request](./field-selection.md):

```json
{{#include ../../../../ndc-reference/tests/query/get_max_article/request.json:1 }}
{{#include ../../../../ndc-reference/tests/query/get_max_article/request.json:3: }}
```

Again, the response is sent in the `__value` field:

```json
{{#include ../../../../ndc-reference/tests/query/get_max_article/expected.json }}
```