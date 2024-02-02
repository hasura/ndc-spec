# Getting Started

The reference implementation will serve queries and mutations based on in-memory data read from newline-delimited JSON files.

First, we will define some types to represent the data in the newline-delimited JSON files. Rows of JSON data will be stored in memory as ordered maps:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:row-type}}
```

Our application state will consist of collections of various types of rows:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:app-state}}
```

In our `main` function, the data connector reads the initial data from the newline-delimited JSON files, and creates the `AppState`:

```rust,no-run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:init_app_state}}
```

Finally, we start a web server with the endpoints which are required by this specification:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:main}}
```

_Note_: the application state is stored in an `Arc<Mutex<_>>`, so that we can perform locking reads and writes in multiple threads.

In the next chapters, we will look at the implementation of each of these endpoints in turn.