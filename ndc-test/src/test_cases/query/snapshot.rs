use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use crate::configuration::TestConfiguration;
use crate::connector::Connector;
use crate::error::Error;
use crate::error::Result;
use crate::snapshot::snapshot_test;

use ndc_client::models::{self};

pub async fn execute_and_snapshot_query<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    query_request: models::QueryRequest,
) -> Result<models::QueryResponse> {
    use std::hash::Hash;

    let request_json = serde_json::to_string_pretty(&query_request).map_err(Error::SerdeError)?;
    let response = connector.query(query_request).await?;

    if let Some(snapshots_dir) = &configuration.snapshots_dir {
        let mut hasher = DefaultHasher::new();
        request_json.hash(&mut hasher);
        let hash = hasher.finish();

        let snapshot_subdir = {
            let mut builder = snapshots_dir.clone();
            builder.extend(vec!["query", format!("{:x}", hash).as_str()]);
            builder
        };

        snapshot_test(snapshot_subdir.join("expected.json").as_path(), &response)?;

        std::fs::write(snapshot_subdir.join("request.json").as_path(), request_json)
            .map_err(Error::CannotOpenSnapshotFile)?;
    }

    Ok(response)
}
