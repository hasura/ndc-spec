use crate::connector::Connector;
use crate::error::{Error, Result};
use crate::reporter::Reporter;
use crate::test;
use ndc_models as models;

pub async fn test_capabilities<C: Connector, R: Reporter>(
    connector: &C,
    reporter: &mut R,
) -> Option<models::CapabilitiesResponse> {
    let capabilities = test!(
        "Fetching /capabilities",
        reporter,
        connector.get_capabilities()
    )?;

    let _ = test!("Validating capabilities", reporter, {
        async { validate_capabilities(&capabilities) }
    });

    Some(capabilities)
}

pub fn validate_capabilities(capabilities: &models::CapabilitiesResponse) -> Result<()> {
    let pkg_version = env!("CARGO_PKG_VERSION");
    let spec_version = semver::VersionReq::parse(format!("^{pkg_version}").as_str())?;
    let claimed_version = semver::Version::parse(capabilities.version.as_str())?;
    if !spec_version.matches(&claimed_version) {
        return Err(Error::IncompatibleSpecification(
            claimed_version,
            spec_version,
        ));
    }

    Ok(())
}
