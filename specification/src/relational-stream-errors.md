# Relational streaming errors

`POST /query/relational/stream` emits one positional JSON array per row. A
connector that advertises `capabilities.relational_query.disjunct_streaming_error`
emits terminal errors as the canonical `RelationalQueryStreamError` object:

```json
{ "type": "error", "status": 500, "message": "Internal error", "details": null }
```

The `type` tag must be `error`, `status` an integer HTTP error code from 400 to
599, and `message` a string. `details` may contain any JSON value; missing details
are read as null. After an error the connector ends the stream. HTTP status is
200 once streaming starts, so consumers must inspect each line, including an
error before the first row. Objects nested in row arrays remain ordinary data.

The canonical Rust model lives in `ndc-models/src/relational_query/types.rs` and
its Draft 7 JSON Schema is `relational_query_stream_error.jsonschema`. SDKs must
consume or generate this model rather than define independent frame shapes.
Without the capability, the legacy `[status, message]` array format is unchanged
and remains ambiguous with valid rows such as `[500, "text"]`.

Capabilities may be captured in persisted build metadata. Consumers must select
the decoder from the matching connector's capability. Deploy readers first, then
coordinate connector upgrades with capability refresh and build activation. An
old connector with stale new-format metadata can have a real error decoded as a
row. Pause streaming or disable it during this transition and during rollback;
restore matching connector and capability state before resuming.
