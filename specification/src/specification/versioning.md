# Versioning

This specification is versioned using semantic versioning, and a data connector declares the [semantic version](https://semver.org) of the specification that it implements via its [capabilities](capabilities.md) endpoint.

Non-breaking changes to the specification may be achieved via the addition of new capabilities, which a connector will be assumed not to implement if the corresponding field is not present in its capabilities endpoint.

## Requirements

The client _may_ send a semantic version string in the `X-Hasura-NDC-Version` HTTP header to any of the HTTP endpoints described by this specification. This header communicates the version of this specification that the client intends to use. Typically this should be the minimum non-breaking version of the specification that is supported by the client, so that the widest range of connectors can be used. For example, if a client sends supports sending v0.1.6 requests, then it technically is sending requests that are compatible with v0.1.0 clients because non-breaking additions are gated behind capabilities and would be disabled for older connectors. In this case, the client should send `0.1.0` as its version in the header.

_If_ the client sends this header, the connector should check compatibility with the requested version, and return an appropriate HTTP error code (e.g. `400 Bad Request`) if it is not capable of providing an implementation. Compatibility is defined as the semver range: `^{requested-version}`. For example, if the client sends `0.2.0`, then the compatible semver range is `^0.2.0`. If the connector implemented spec version `0.1.6`, this would be incompatible, but if it implemented spec version `0.2.1`, this would be compatible.

_Note_: the `/capabilities` endpoint also indicates the implemented specification version for any connector, but it may not be practical for a client to check the capabilities endpoint before issuing a new request, so this provides a way to check compatibility in the course of a normal request.
