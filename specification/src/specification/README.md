# API Specification

| Version |
|---------|
| `0.1.0`   |

A data connector encapsulates a data source by implementing the protocol in this specification.

A data connector must implement several web service endpoints:

- A __capabilities__ endpoint, which describes which features the data source is capable of implementing.
- A __schema__ endpoint, which describes the resources provided by the data source, and the shape of the data they contain.
- A __query__ endpoint, which reads data from one of the relations described by the schema endpoint.
- A __query/explain__ endpoint, which explains a query plan, without actually executing it.
- A __mutation__ endpoint, which modifies the data in one of the relations described by the schema endpoint.
- A __mutation/explain__ endpoint, which explains a mutation plan, without actually executing it.
- A __metrics__ endpoint, which exposes runtime metrics about the data connector.
