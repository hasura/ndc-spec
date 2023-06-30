# Versioning

This specification is versioned using semantic versioning, and a data connector claims compatibility with a [semantic version](https://semver.org) range via its [capabilities](capabilities.md) endpoint.

Non-breaking changes to the specification may be achieved via the addition of new capabilities, which a connector will be assumed not to implement if the corresponding field is not present in its capabilities endpoint.