# Runtime version checks

## Purpose

It is quite easy to accidentally update a connector but forget to refresh its `DataConnectorLink` (which contains the NDC version, schema and capabilities), which means the engine sends the wrong version of the request to the connector. This can lead to subtle bugs.

By having an explicit version assertion, connector can immediately reject a request that is for an incorrect version.

## Proposal

Add a `X-Hasura-NDC-Version` HTTP header to all requests, which the client _may_ send in order to clarify the intended protocol version using a semantic version string. The connector should check any provided version and fail fast with a HTTP error code.
