use crate::configuration::TestConfiguration;
use crate::connector::Connector;
use crate::error::{Error, Result};
use crate::reporter::{Reporter, ReporterExt};
use crate::results::TestResults;
use crate::snapshot::snapshot_test;
use ndc_client::models;
use std::cell::RefCell;

pub async fn test_capabilities<C: Connector, R: Reporter>(
    configuration: &TestConfiguration,
    connector: &C,
    reporter: &R,
    results: &RefCell<TestResults>,
) -> Option<models::CapabilitiesResponse> {
    let capabilities = reporter
        .test("Fetching /capabilities", results, async {
            let response: models::CapabilitiesResponse = connector.get_capabilities().await?;
            for snapshots_dir in configuration.snapshots_dir.iter() {
                snapshot_test(snapshots_dir.join("capabilities").as_path(), &response)?;
            }
            Ok(response)
        })
        .await?;

    let _ = reporter
        .test("Validating capabilities", results, async {
            validate_capabilities(&capabilities)
        })
        .await;

    Some(capabilities)
}

pub fn validate_capabilities(capabilities: &models::CapabilitiesResponse) -> Result<()> {
    let pkg_version = env!("CARGO_PKG_VERSION");
    let spec_version = semver::VersionReq::parse(format!("^{}", pkg_version).as_str())?;
    let claimed_version = semver::Version::parse(capabilities.version.as_str())?;
    if !spec_version.matches(&claimed_version) {
        return Err(Error::IncompatibleSpecification(claimed_version, spec_version));
    }

    Ok(())
}
