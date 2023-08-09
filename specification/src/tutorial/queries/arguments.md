# Evaluating Arguments

Now that we have reduced the problem to a single set of query variables, we must evaluate any [collection arguments](../../specification/queries/arguments.md), and in turn, evaluate the _collection_ of rows that we will be working with. 

From there, we will be able to apply predicates, sort and paginate rows. But one step at a time!

The first step is to evaluate each argument, which the `execute_query_with_variables` function does by delegating to the `eval_argument` function:

```rust,no_run,noplayground
{{#include ../../../../ndc-reference/bin/reference/main.rs:execute_query_with_variables}}
```

Once this is complete, and we have a collection of evaluated `argument_values`, we can delegate to the `get_collection_by_name` function. This function peforms the work of computing the full collection, by pattern matching on the name of the collection:

```rust,no_run,noplayground
{{#include ../../../../ndc-reference/bin/reference/main.rs:get_collection_by_name}}
```

_Note 1_: the `articles_by_author` collection is the only example here which has to apply any arguments. It is provided as an example of a collection which accepts an `author_id` argument, and it must validate that the argument is present, and that it is an integer.

_Note 2_: the `latest_article_id` collection is provided as an example of a [function](../../specification/schema/functions.md). It is a collection like all the others, but must follow the rules for functions: it must consist of a single row, with a single column named `__value`.

In the next section, we will break down the implementation of `execute_query`.
Once we have computed the full collection, we can move onto evaluating the query in the context of that collection, using the `execute_query` function:

```rust,no_run,noplayground
{{#include ../../../../ndc-reference/bin/reference/main.rs:execute_query_signature}}
```

In the next section, we will break down the implementation of `execute_query`.