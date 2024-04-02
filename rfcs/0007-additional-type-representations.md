# Additional Type Representations

## Purpose

The current list of scalar type representations is inadequate:

- JSON integer and number types are not what we want for most applications, since they are represented by an unusual 53-bit number type. Since scalars can be both inputs and outputs, and are therefore invariant, we cannot safely convert to and from larger and smaller integer and floating point types.
- We want to be able to generate some standard scalar types in Hasura for types which are used across different data sources, e.g. UUIDs. This needs support in NDC.

## Proposal

Add the following type representations to `ndc_models::TypeRepresentation`:

|Name|Description|JSON representation|
|-|-|-|
| Int8 | A 8-bit signed integer with a minimum value of -2^7 and a maximum value of 2^7 - 1 | Number |
| Int16 | A 16-bit signed integer with a minimum value of -2^15 and a maximum value of 2^15 - 1 |Number |
| Int32 | A 32-bit signed integer with a minimum value of -2^31 and a maximum value of 2^31 - 1 |Number |
| Int64 | A 64-bit signed integer with a minimum value of -2^63 and a maximum value of 2^63 - 1 |String |
| Float32 | An IEEE-754 single-precision floating-point number | Number |
| Float64 | An IEEE-754 double-precision floating-point number | String |
| Decimal | | String |
| Char | A unicode scalar value |
| Uuid | | String |
| Date | | String |
| Timestamp | | String |
| TimestampTZ | | String (ISO 8601) |
| Bytes | | Base64-encoded String |
| Geography | Geometry | String (GeoJSON) |

## Open Questions

- `Decimal`: what about precision?
- Need regexes for date/timestamp(tz)

## Alternatives considered

### Open world of scalar representations

Instead of standardizing a closed list of scalars, we could enable an open world of scalars by adding a single scalar identified by a URL. The URL would point to documentation about that scalar representation. Such a representation could be special-cased by any client recognizing the URL.

Hasura could publish a list of supported scalars.

#### Cons

- This would lead to an indirect dependency between connectors and engine.
- It violates the dependency-inversion principle.
- It becomes harder to later allow an open world approach to other aspects, e.g. data transport. If we allow other transports later, then every scalar representation needs to define its representation for every possible transport.