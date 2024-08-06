# Service Health

Data connectors should provide __health endpoints__ which can be used to indicate service health and readiness to any client applications.

On success, these endpoints return `200 OK` and no body. On failure, they return an error as described in [Error Handling](./error-handling.md).

## Readiness

"Readiness" is defined as the ability to accept a request. It does not require the service to be able to handle the request.

In the case of a connector, it might not be "ready" if, for example, it has not finished initializing its state.

Orchestrators (such as Kubernetes) can use this endpoint to determine whether they can start routing requests to this connector.

It's recommended that the connector does not start listening on a socket until it has finished initializing and is "ready", but there might be good reasons to start listening earlier.

The connector should not need to make any requests to other services in order to handle this request.

### Request

```
GET /health/ready
```

`GET /health` and `GET /` are aliases for the readiness health endpoint.

### Response

If the data connector is "ready", then the endpoint should return status code `200 OK`.

Otherwise, it should ideally return a status code `503 Service Unavailable`, or some other appropriate HTTP error code, along with JSON representing an error.

## Liveness

"Liveness" is defined as the ability to handle a request internally. It assumes readiness, and also a consistent state. For example, if the server is in a deadlocked state, or a connection pool is broken with no ability to automatically recover, it might be "ready" but not "live".

Orchestrators (such as Kubernetes) can use this endpoint to determine whether the connector needs to be restarted or replaced.

The connector should not need to make any requests to other services in order to handle this request.

### Request

```
GET /health/live
```

### Response

If the data connector is "live", then the endpoint should return status code `200 OK`.

Otherwise, it should ideally return a status code `500 Internal Server Error`, or some other appropriate HTTP error code, along with JSON representing an error.

## Connectedness

"Connectedness" is defined as the ability to communicate with any backing services. It assumes readiness and almost always liveness, and also a valid connection.

Because this exposes information about connectivity across services, it can be used for automated monitoring or alerting, but it is unlikely that automated decisions can be taken based on its response.

Unlike the readiness and liveness checks, connectedness checks might make requests to other services, and so may not return an immediate response.

### Request

```
GET /health/connected
```

### Response

If the data connector is "connected", then the endpoint should return status code `200 OK`.

Otherwise, it should ideally return a status code `502 Bad Gateway` if the connection is invalid, `504 Gateway Timeout` if a connection cannot be made, or some other appropriate HTTP error code, along with JSON representing an error.
