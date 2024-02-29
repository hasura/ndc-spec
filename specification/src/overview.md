# Overview

<div class="warning">

<b>NOTE</b>

This specification contains the low-level details for connector authors, and is intended as a complete reference.

Users looking to build their own connectors might want to also look at some additional resources:

- [Hasura Connector Hub](https://hasura.io/connectors) contains a list of currently available connectors
- [Let's Build a Connector](https://hasura.io/learn/graphql/hasura-v3-ts-connector/introduction/) is a step-by-step to creating a connector using TypeScript
</div>

---

Hasura data connectors allow you to extend the functionality of the Hasura server by providing web services which can resolve new sources of data. By following this specification, those sources of data can be added to your Hasura graph, and the usual Hasura features such as relationships and permissions will be supported for your data source.

This specification is designed to be as general as possible, supporting many different types of data source, while still being targeted enough to provide useful features with high performance guarantees. It is important to note that data connectors are designed for tabular data which supports efficient filtering and sorting. If you are able to model your data source given these constraints, then it will be a good fit for a data connector, but if not, you might like to consider a GraphQL remote source integration with Hasura instead.