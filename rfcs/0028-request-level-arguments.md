# Request-level arguments in NDC

# Motivation

Users have requested the following features:

- Dynamically update connection settings per request given user’s session variables
- Request that a given request access the write instance rather than any read replica

We plan to enable this with a `pre-ndc-request-plugin` type that could decorate an NDC request with additional arguments given various session variables. This allows the user to provide the session variables \-\> connection settings mapping themselves without needing to bake this logic into individual data connectors. An example for `ndc-postgres` is included below.

It could also be useful for:

- Header forwarding for NDC Lambda connector \- currently this works with collection / mutation arguments and it’s very unwieldy
- Dynamic connection settings unlock users where they run multiple homogeneous DBs for each of their own customers’ data, and wish to switch between them without deploying an NDC connector for each customer.

# Design

A data connector would advertise any request-level arguments with a new optional section in `/schema` like this:

```rust
pub struct RequestLevelArguments {
  /// Any arguments that all Query requests require
  pub query_arguments: BTreeMap<ArgumentName, ArgumentInfo>,
  /// Any arguments that all Mutation requests require
  pub mutation_arguments: BTreeMap<ArgumentName, ArgumentInfo>,
  /// Any arguments that all Relational Query requests require
  pub relational_query_arguments: BTreeMap<ArgumentName, ArgumentInfo>,
}
```

`ArgumentInfo` is the same type used for other argument definitions \- it includes a type and optional description. If a nullable argument is not supplied, we shall assume a `null` value has been passed for that argument.

The following fields are added to the `QueryRequest`, `MutationRequest` and `RelationalQuery` types:

```rust
/// Values to be provided to request-level arguments.
pub request_arguments: Option<BTreeMap<ArgumentName, serde_json::Value>>,
```

Any arguments advertised in the schema for this type of request must be provided for the request to succeed.

# Best practices for auth

Let’s look at how we’d implement dynamic connection strings in `ndc-postgres`.

Firstly, we’d add a `dynamicConnections` option to the `ndc-postgres` configuration. When enabled, this would include a `connection_identifier` request-level argument for queries and mutations, with type `String!`. This would be visible for both `query_arguments` and `mutation_arguments` when calling the `/schema` endpoint.

When deploying `ndc-postgres` , we’d include a mapping from `connection_identifier` \-\> `connection_string` as an environment variable called `HASURA_POSTGRES_CONNECTION_MAPPINGS`. This would be a JSON string with a format the connector would check on startup and this would most likely be populated from a secret.

Let’s say ours looks like:

```json
{
  "read_replica": "postgresql://read@database",
  "main_replica": "postgresql://write@database"
}
```

An NDC request comes in with the request-level argument `connection_identifier` set to `read_replica`. `ndc-postgres` looks it up in this map, finds “`postgresql://read@database`” and uses that connection string to serve the request.

If another request comes in with a `connection_identifier` of `main_replace`, we’d use “`postgresql::/write@database.com`” instead.

# Alternative: use HTTP headers

- Avoided because we want to define these arguments with types in the schema, and we already have a bunch of implicitly defined headers in use in various connectors that we’d need to either ignore or special case.

# Alternative: use collection / mutation arguments

- They are very fiddly to use, and any argument needs exposing in the schema for each individual collection / command.
- We have to rely on convention to ensure the arguments are the same for each collection / command.
- We also want something less expressive, so that users cannot reference `foreach` variables or provide a different value per mutation.

# Alternative: fork connectors that need special connection behaviour

- This will mean maintaining and making releases for long-running forks to patch security issues.
- That work needs repeating in every connector that wants this behaviour

# Risks

- Does this feature encourage users to include (for example) connection strings in the request body where they may get logged?
  - Can we mitigate this risk by encouraging best practices, such as not logging the whole request body, or passing secrets store keys instead?
  - Given we already encourage users to forward headers via regular command/collection arguments, is this any more of an issue than it already was?
