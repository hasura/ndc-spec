# Versioning

This specification is versioned using semantic versioning, and a data connector claims compatibility with a [semantic version](https://semver.org) range via its [capabilities](capabilities.md) endpoint.

Non-breaking changes to the specification may be achieved via the addition of new capabilities, which a connector will be assumed not to implement if the corresponding field is not present in its capabilities endpoint.

## Requirements

The client _may_ send a semantic version string in the `X-Hasura-NDC-Version` HTTP header to any of the HTTP endpoints described by this specification. This header communicates the version of this specification that the client intends to use.

_If_ the client sends this header, the connector should check compatibility with the requested version, and return an appropriate HTTP error code (e.g. `400 Bad Request`) if it is not capable of providing an implementation.

_Note_: the `/capabilities` endpoint also indicates the implemented specification version for any connector, but it may not be practical for a client to check the capabilities endpoint before issuing a new request, so this provides a way to check compatibility in the course of a normal request.