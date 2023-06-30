# Basics

Data connectors are implemented as HTTP services. To refer to a running data connector, it suffices to specify its base URL. All required endpoints are specified relative to this base URL.

All endpoints should accept JSON (in the case of POST request bodies) and return JSON using the `application/json` content type. The particular format of each JSON document will be specified for each endpoint.