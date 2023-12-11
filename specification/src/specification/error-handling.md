# Error Handling

## Status Codes

Data connectors should use standard HTTP error codes to signal error conditions back to the Hasura server. In particular, the following error codes should be used in the indicated scenarios:

| Response Code | Meaning | Used when |
|-|-|-|
| 200 | OK | The request was handled successfully according to this specification .|
| 400 | Bad Request | The request did not match the data connector's expectation based on this specification. |
| 403 | Forbidden | The request could not be handled because a permission check failed - for example, a mutation might fail because a check constraint was not met. |
| 409 | Conflict | The request could not be handled because it would create a conflicting state for the data source - for example, a mutation might fail because a foreign key constraint was not met. |
| 422 | Unprocessable Content | The request could not be handled because, while the request was well-formed, it was not semantically correct. For example, a value for a custom scalar type was provided, but with an incorrect type. |
| 500 | Internal Server Error | The request could not be handled because of an error on the server |
| 501 | Not Supported | The request could not be handled because it relies on an unsupported [capability](capabilities.md). _Note_: this ought to indicate an error on the _caller_ side, since the caller should not generate requests which are incompatible with the indicated capabilities. |
| 502 | Bad Gateway | The request could not be handled because an upstream service was unavailable or returned an unexpected response, e.g., a connection to a database server failed |

## Response Body

Data connectors should return an [`ErrorResponse`](../reference/types.md#errorresponse) as JSON in the response body, in the case of an error. 