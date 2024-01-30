# NDC Versioning Specification

## Purpose

This document exists to specify the way in which this NDC specification ought to be versioned, how connectors can claim compatibility at a given version, and how clients can depend on it.

## Proposal

- The NDC specification will be versioned using semantic versioning.
  - Breaking changes will be determined by what is breaking at the level of the NDC protocol itself. That is, a change is non-breaking if, for all possible connectors, implementing the specification correctly at the previous version entails implementing the specification correctly at the new version.
- Released versions will be provided as tags at the [public GitHub repository](https://github.com/hasura/ndc-spec).
- The specification will maintain a changelog page, which indicates major changes between releases.
- A connector will claim compatibility with a _single_ semantic version of the specification by specifying that version in the `version` field of the capabilities response.
  - Note: previously, we specified a version range, but there is little point claiming compatibility with an older version, once support for a newer version is announced; in addition, non-contiguous version ranges do not make sense, so it makes sense to identify the single most-recent compatible version.
  - For clarity: if a connector claims compatibility with a particular version, then any client should be able to use this connector, as long as the client is using a specification version which is compatible with the connector specification version, according to the semantic versioning specification.
- A client can depend on a particular specification version by either:
  - using the Rust client library provided in this repository, at the same _Git revision_.
    - Note: the semantic versioning of the repository tracks the _specification version_, and not the Rust library versions. While we will try to not introduce unnecessary breaking changes between major specification revisions, we can't guarantee that, and the user should be careful to depend on these Rust libraries at the correct version.
  - using the generated JSON schema files at the same _Git revision_.

## Changes Required

- The `versions` field in the capability response needs to be replaced with a `version` field.