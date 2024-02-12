# Procedures

The `execute_procedure` function is responsible for executing a single procedure:

```rust,no_run,noplayground
{{#include ../../../../ndc-reference/bin/reference/main.rs:execute_procedure_signature}}
```

The function receives the application `state`, along with the `name` of the procedure to invoke, a list of `arguments`, a list of `fields` to return, and a list of `collection_relationships`.

The function matches on the name of the procedure, and fails if the name is not recognized. We will walk through each procedure in turn.

```rust,no_run,noplayground
{{#include ../../../../ndc-reference/bin/reference/main.rs:execute_procedure_signature_impl}}
```

## `upsert_article`

The `upsert_article` procedure is implemented by the `execute_upsert_article` function.

The `execute_upsert_article` function reads the `article` argument from the `arguments` list, failing if it is not found or invalid.

It then inserts or updates that article in the application state, depending on whether or not an article with that `id` already exists or not.

Finally, it delegates to the `eval_nested_field` function to evaluate any nested fields, and returns the selected fields in the result:

```rust,no_run,noplayground
{{#include ../../../../ndc-reference/bin/reference/main.rs:execute_upsert_article}}
```

## `delete_articles`

The `delete_articles` procedure is implemented by the `execute_delete_articles` function.

It is provided as an example of a procedure with a [predicate type](../../specification/types.md#predicate-types) as the type of an argument.

The `execute_delete_articles` function reads the `where` argument from the `arguments` list, failing if it is not found or invalid.

It then deletes all articles in the application state which match the predicate, and returns a list of the deleted rows.

This function delegates to the `eval_nested_field` function to evaluate any nested fields, and returns the selected fields in the result:

```rust,no_run,noplayground
{{#include ../../../../ndc-reference/bin/reference/main.rs:execute_delete_articles}}
```