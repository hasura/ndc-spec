# Remove `ndc-client`

## Purpose

`ndc-models` has been extracted from `ndc-client`. `ndc-client` now only contains the HTTP client library for the NDC methods.

However, there is not a good one-size-fits-all client for all applications. Indeed, the current client library already contains some details which are overfitted to the needs of Hasura V3 engine (e.g. tracing) and we are looking to add more (streaming responses).

## Proposal

- Move the code in `ndc-client` into `ndc-test` as a crate-private module (its only consumer in this repository)
  - Remove any tracing-specific code from this client
- V3 engine should build a separate client which implements tracing, streaming, etc.