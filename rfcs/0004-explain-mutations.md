# Explain for Mutations

## Purpose

We have `/expain` for queries, but not for mutations. It's equally useful for debugging, but even more useful for testing - while we can execute queries fearlessly, we can't modify the user's data source arbitrarily in `ndc-test`. But we _can_ snapshot test the explain output, for both queries and mutations.

## Proposal 

- Rename `/explain` to `/query/explain`
- Add `/mutation/explain` which takes a `MutationRequest` and returns an `ExplainResponse`.
- Rename capabilities accordingly: `query.explain` and `mutation.explain`.
- Later: add explain snapshot testing to `ndc-test`.