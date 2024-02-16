use std::{fs::File, path::Path};

use crate::error::Error;

pub fn snapshot_test<R>(snapshot_path: &Path, expected: &R) -> Result<(), Error>
where
    R: serde::Serialize + serde::de::DeserializeOwned + PartialEq,
{
    if snapshot_path.exists() {
        let snapshot_file = File::open(snapshot_path).map_err(Error::CannotOpenSnapshotFile)?;
        let snapshot: R = serde_json::from_reader(snapshot_file).map_err(Error::SerdeError)?;

        if snapshot != *expected {
            let expected_json =
                serde_json::to_string_pretty(&expected).map_err(Error::SerdeError)?;
            return Err(Error::ResponseDidNotMatchSnapshot(
                snapshot_path.into(),
                expected_json,
            ));
        }
    } else {
        let parent = snapshot_path.parent().unwrap();
        let snapshot_file = (|| {
            std::fs::create_dir_all(parent)?;
            File::create(snapshot_path)
        })()
        .map_err(Error::CannotOpenSnapshotFile)?;

        serde_json::to_writer_pretty(snapshot_file, &expected).map_err(Error::SerdeError)?;
    }

    Ok(())
}
