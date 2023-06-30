# Service Health

Data connectors should provide a __health endpoint__ which can be used to indicate service health and readiness to any client applications.

## Request

```
GET /healthz
```

## Response

If the data connector is available and ready to accept requests, then the health endpoint should return status code `204 No Content`.

Otherwise, it should ideally return a status code `503 Service Unavailable`, or some other appropriate HTTP error code.