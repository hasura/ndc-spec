pub type Result<A> = std::result::Result<A, Error>;

pub type OtherError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error communicating with the connector: {0}")]
    CommunicationError(#[from] super::client::Error),
    #[error("error generating test data: {0}")]
    StrategyError(#[from] rand::Error),
    #[error("error parsing semver range: {0}")]
    SemverError(#[from] semver::Error),
    #[error(
        "capabilities.version ({0}) is not compatible with the current version of the specification ({1})"
    )]
    IncompatibleSpecification(semver::Version, semver::VersionReq),
    #[error("collection {0} is not a defined collection")]
    CollectionIsNotDefined(ndc_models::CollectionName),
    #[error("collection type {0} is not a defined object type")]
    CollectionTypeIsNotDefined(ndc_models::ObjectTypeName),
    #[error("named type {0} is not a defined object or scalar type")]
    NamedTypeIsNotDefined(ndc_models::TypeName),
    #[error("object type {0} is not a defined object or scalar type")]
    ObjectTypeIsNotDefined(ndc_models::ObjectTypeName),
    #[error("field {0} is not defined on object type")]
    FieldIsNotDefined(ndc_models::FieldName),
    #[error("relationship {0} is not defined in request")]
    RelationshipIsNotDefined(ndc_models::RelationshipName),
    #[error("expected null rows in RowSet")]
    RowsShouldBeNullInRowSet,
    #[error("expected non-null rows in RowSet")]
    RowsShouldBeNonNullInRowSet,
    #[error("expected null aggregates in RowSet")]
    AggregatesShouldBeNullInRowSet,
    #[error("expected non-null aggregates in RowSet")]
    AggregatesShouldBeNonNullInRowSet,
    #[error("expected {0} RowSet(s) in the QueryResponse, got {1}")]
    UnexpectedRowsets(usize, usize),
    #[error("expected RowSet in response for field {0}")]
    ExpectedRowSet(ndc_models::FieldName),
    #[error("expected <= {0} rows in RowSet, got {1}")]
    TooManyRowsInResponse(u32, u32),
    #[error("expected non-empty rows in RowSet")]
    ExpectedNonEmptyRows,
    #[error("requested field {0} was missing in response")]
    MissingField(ndc_models::FieldName),
    #[error("field {0} was not expected in response")]
    UnexpectedField(ndc_models::FieldName),
    #[error("scalar type {0} has multiple equality operators")]
    MultipleEqualityOperators(ndc_models::ScalarTypeName),
    #[error("error response from connector: {0:?}")]
    ConnectorError(ndc_models::ErrorResponse),
    #[error("cannot open snapshot file: {0:?}")]
    CannotOpenSnapshotFile(std::io::Error),
    #[error("error (de)serializing data structure: {0:?}")]
    SerdeError(#[from] serde_json::Error),
    #[error("snapshot did not match file {0}: {1}")]
    ResponseDidNotMatchSnapshot(std::path::PathBuf, String),
    #[error("cannot open benchmark directory: {0:?}")]
    CannotOpenBenchmarkDirectory(std::io::Error),
    #[error("cannot open benchmark report: {0:?}")]
    CannotOpenBenchmarkReport(std::io::Error),
    #[error("benchmark deviated significantly ({0:.02}σ) from previous result")]
    BenchmarkExceededTolerance(f64),
    #[error("response from connector does not satisfy requirement: {0}")]
    ResponseDoesNotSatisfy(String),
    #[error("invalid response at path {}: expected {1}", .0.join("."))]
    InvalidValueInResponse(Vec<String>, String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("other error: {0}")]
    OtherError(#[from] OtherError),
}
