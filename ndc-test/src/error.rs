use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("error communicating with the connector: {0}")]
    CommunicationError(#[from] ndc_client::apis::Error),
    #[error("error generating test data: {0}")]
    StrategyError(rand::Error),
    #[error("error parsing semver range: {0}")]
    SemverError(#[from] semver::Error),
    #[error(
        "capabilities.version ({0}) is not compatible with the current version of the specification ({1})"
    )]
    IncompatibleSpecification(semver::Version, semver::VersionReq),
    #[error("collection {0} is not a defined collection")]
    CollectionIsNotDefined(String),
    #[error("collection type {0} is not a defined object type")]
    CollectionTypeIsNotDefined(String),
    #[error("named type {0} is not a defined object or scalar type")]
    NamedTypeIsNotDefined(String),
    #[error("object type {0} is not a defined object or scalar type")]
    ObjectTypeIsNotDefined(String),
    #[error("insertable column {0} is not defined on collection type")]
    InsertableColumnNotDefined(String),
    #[error("updatable column {0} is not defined on collection type")]
    UpdatableColumnNotDefined(String),
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
    ExpectedRowSet(String),
    #[error("expected <= {0} rows in RowSet, got {1}")]
    TooManyRowsInResponse(u32, u32),
    #[error("expected non-empty rows in RowSet")]
    ExpectedNonEmptyRows,
    #[error("requested field {0} was missing in response")]
    MissingField(String),
    #[error("field {0} was not expected in response")]
    UnexpectedField(String),
    #[error("error response from connector: {0:?}")]
    ConnectorError(ndc_client::models::ErrorResponse),
    #[error("cannot open snapshot file: {0:?}")]
    CannotOpenSnapshotFile(std::io::Error),
    #[error("error (de)serializing data structure: {0:?}")]
    SerdeError(serde_json::Error),
    #[error("snapshot did not match file {0}: {1}")]
    ResponseDidNotMatchSnapshot(PathBuf, String),
    #[error("cannot open benchmark directory: {0:?}")]
    CannotOpenBenchmarkDirectory(std::io::Error),
    #[error("cannot open benchmark report: {0:?}")]
    CannotOpenBenchmarkReport(std::io::Error),
    #[error("benchmark deviated significantly ({0:.02}σ) from previous result")]
    BenchmarkExceededTolerance(f64),
    #[error("response from connector does not satisfy requirement: {0}")]
    ResponseDoesNotSatisfy(String),
    #[error("other error")]
    OtherError(#[from] Box<dyn std::error::Error>),
}

impl From<rand::Error> for Error {
    fn from(value: rand::Error) -> Self {
        Error::StrategyError(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::SerdeError(value)
    }
}

impl From<ndc_client::apis::Error> for Box<Error> {
    fn from(value: ndc_client::apis::Error) -> Self {
        Box::new(Error::CommunicationError(value))
    }
}

impl From<semver::Error> for Box<Error> {
    fn from(value: semver::Error) -> Self {
        Box::new(Error::SemverError(value))
    }
}

impl From<Box<dyn std::error::Error>> for Box<Error> {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Box::new(Error::OtherError(value))
    }
}

pub type Result<A> = core::result::Result<A, Error>;
