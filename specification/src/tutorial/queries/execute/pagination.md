# Pagination

Once the irrelevant rows have been filtered out, the `execute_query` function applies the `limit` and `offset` arguments by calling the `paginate function:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:execute_query_paginate}}
```

The `paginate` function is implemented using the `skip` and `take` functions on iterators:

```rust,no_run,noplayground
{{#include ../../../../../ndc-reference/bin/reference/main.rs:paginate}}
```