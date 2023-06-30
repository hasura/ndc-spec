# Getting Started

The reference implementation will serve queries and mutations based on in-memory data read from CSV files.

First, we will define some types to represent the data in the CSV files. The structure of these types will also be reflected in our data connector's schema: 

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:csv-types}}
```

Our data connector's application state will consist of collections of each of these types:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:app-state}}
```

In our `main` function, the data connector reads the initial data from the CSV files, and creates the `AppState`, before starting a web server with the required endpoints:

```rust,no_run,noplayground
{{#include ../../../ndc-reference/bin/reference/main.rs:main}}
```

In the next chapters, we will look at the implementation of each of these endpoints in turn.