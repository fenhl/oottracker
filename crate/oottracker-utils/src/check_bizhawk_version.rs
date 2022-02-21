#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        cmp::Ordering::*,
        time::Duration,
    },
    semver::Version,
    tokio::fs,
    oottracker_utils::version,
};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)] BizHawkVersionCheck(#[from] version::BizHawkError),
    #[error(transparent)] InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error(transparent)] Io(#[from] std::io::Error),
    #[error(transparent)] Reqwest(#[from] reqwest::Error),
    #[error("BizHawk is outdated ({local} installed, {latest} available)")]
    BizHawkOutdated {
        latest: Version,
        local: Version,
    },
    #[error("locally installed BizHawk is newer than latest release")]
    BizHawkVersionRegression,
}

#[wheel::main]
async fn main() -> Result<(), Error> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!("token {}", fs::read_to_string("assets/release-token").await?))?);
    headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static(concat!("oottracker-release/", env!("CARGO_PKG_VERSION"))));
    let client = reqwest::Client::builder()
        .user_agent(concat!("oottracker/", env!("CARGO_PKG_VERSION")))
        .default_headers(headers)
        .timeout(Duration::from_secs(600))
        .http2_prior_knowledge()
        .use_rustls_tls()
        .https_only(true)
        .build()?;
    let [major, minor, patch, _] = oottracker_bizhawk::bizhawk_version();
    let local_version = Version::new(major.into(), minor.into(), patch.into());
    let remote_version = version::bizhawk_latest(&client).await?;
    match local_version.cmp(&remote_version) {
        Less => Err(Error::BizHawkOutdated { local: local_version, latest: remote_version }),
        Equal => Ok(()),
        Greater => Err(Error::BizHawkVersionRegression),
    }
}
