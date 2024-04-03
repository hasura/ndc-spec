use std::{
    collections::hash_map::DefaultHasher,
    fs::File,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use ndc_models as models;

use crate::{
    connector::Connector,
    error::{Error, Result},
};

pub struct SnapshottingConnector<'a, C: Connector> {
    pub snapshot_path: &'a PathBuf,
    pub connector: &'a C,
}

#[async_trait(?Send)]
impl<'a, C: Connector> Connector for SnapshottingConnector<'a, C> {
    async fn get_capabilities(&self) -> Result<models::CapabilitiesResponse> {
        let response: models::CapabilitiesResponse = self.connector.get_capabilities().await?;
        snapshot_test(self.snapshot_path.join("capabilities").as_path(), &response)?;
        Ok(response)
    }

    async fn get_schema(&self) -> Result<models::SchemaResponse> {
        let response = self.connector.get_schema().await?;
        snapshot_test(self.snapshot_path.join("schema").as_path(), &response)?;
        Ok(response)
    }

    async fn query(&self, request: models::QueryRequest) -> Result<models::QueryResponse> {
        let request_json = serde_json::to_string_pretty(&request)?;
        let response = self.connector.query(request).await?;

        let mut hasher = DefaultHasher::new();
        request_json.hash(&mut hasher);
        let hash = hasher.finish();

        let snapshot_subdir = {
            let mut builder = self.snapshot_path.clone();
            builder.extend(vec!["query", format!("{hash:x}").as_str()]);
            builder
        };

        snapshot_test(snapshot_subdir.join("expected.json").as_path(), &response)?;

        std::fs::write(snapshot_subdir.join("request.json").as_path(), request_json)
            .map_err(Error::CannotOpenSnapshotFile)?;

        Ok(response)
    }

    async fn mutation(&self, request: models::MutationRequest) -> Result<models::MutationResponse> {
        self.connector.mutation(request).await
    }
}

pub fn snapshot_test<R>(snapshot_path: &Path, expected: &R) -> Result<()>
where
    R: serde::Serialize + serde::de::DeserializeOwned + PartialEq,
{
    if snapshot_path.exists() {
        let snapshot_file = File::open(snapshot_path).map_err(Error::CannotOpenSnapshotFile)?;
        let snapshot: R = serde_json::from_reader(snapshot_file)?;

        if snapshot != *expected {
            let actual = serde_json::to_string_pretty(&expected)?;
            return Err(Error::ResponseDidNotMatchSnapshot(
                snapshot_path.to_path_buf(),
                actual,
            ));
        }
    } else {
        let parent = snapshot_path.parent().unwrap();
        let snapshot_file = (|| {
            std::fs::create_dir_all(parent)?;
            File::create(snapshot_path)
        })()
        .map_err(Error::CannotOpenSnapshotFile)?;

        serde_json::to_writer_pretty(snapshot_file, &expected)?;
    }

    Ok(())
}
