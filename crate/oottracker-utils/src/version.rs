use {
    std::{
        iter,
        process::Stdio,
        str::FromStr as _,
    },
    graphql_client::GraphQLQuery,
    itertools::Itertools as _,
    semver::Version,
    serde::{
        Deserialize,
        Serialize,
    },
    serde_json::Value as Json,
    tokio::process::Command,
};

pub const INFO_PLIST_PATH: &str = "assets/macos/OoT Tracker.app/Contents/Info.plist";

#[derive(Deserialize, Serialize)]
pub struct Plist {
    #[serde(rename = "CFBundleShortVersionString")]
    pub bundle_short_version_string: Version,
    #[serde(rename = "CFBundleVersion")]
    pub bundle_version: String,
    #[serde(flatten)]
    _rest: Json, //HACK see https://github.com/ebarnard/rust-plist/issues/54#issuecomment-653756460
}

#[derive(Debug, thiserror::Error)]
pub enum BizHawkError {
    #[error(transparent)] ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)] Reqwest(#[from] reqwest::Error),
    #[error("no info returned in BizHawk version query response")]
    EmptyResponse,
    #[error("no BizHawk repo info returned")]
    MissingRepo,
    #[error("no releases in BizHawk GitHub repo")]
    NoReleases,
    #[error("the latest BizHawk GitHub release has no name")]
    UnnamedRelease,
}

#[cfg(windows)]
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../assets/graphql/github-schema.graphql",
    query_path = "../../assets/graphql/github-bizhawk-version.graphql",
)]
struct BizHawkVersionQuery;

pub async fn bizhawk_latest(client: &reqwest::Client) -> Result<Version, BizHawkError> {
    let remote_version_string = client.post("https://api.github.com/graphql")
        .bearer_auth(include_str!("../../../assets/release-token"))
        .json(&BizHawkVersionQuery::build_query(biz_hawk_version_query::Variables {}))
        .send().await?
        .error_for_status()?
        .json::<graphql_client::Response<biz_hawk_version_query::ResponseData>>().await?
        .data.ok_or(BizHawkError::EmptyResponse)?
        .repository.ok_or(BizHawkError::MissingRepo)?
        .latest_release.ok_or(BizHawkError::NoReleases)?
        .name.ok_or(BizHawkError::UnnamedRelease)?;
    let (major, minor, patch) = remote_version_string.split('.').map(u64::from_str).chain(iter::repeat(Ok(0))).next_tuple().expect("iter::repeat produces an infinite iterator");
    Ok(Version::new(major?, minor?, patch?))
}

pub async fn check_cli_version(package: &str, version: &Version) {
    let cli_output = String::from_utf8(Command::new("cargo").arg("run").arg(format!("--package={}", package)).arg("--").arg("--version").env("DATABASE_URL", include_str!("../../../assets/web/env.txt").split_once('=').unwrap().1).stdout(Stdio::piped()).output().await.expect("failed to run CLI with --version").stdout).expect("CLI version output is invalid UTF-8");
    let (cli_name, cli_version) = cli_output.trim_end().split(' ').collect_tuple().expect("no space in CLI version output");
    assert_eq!(cli_name, package);
    assert_eq!(*version, cli_version.parse().expect("failed to parse CLI version"));
}

pub async fn version() -> Version {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version");
    assert_eq!(version, plist::from_file::<_, Plist>(INFO_PLIST_PATH).expect("failed to read plist for version check").bundle_short_version_string);
    assert_eq!(version, ootr::version());
    assert_eq!(version, ootr_dynamic::version());
    assert_eq!(version, ootr_static::version()); // also checks ootr-static-derive
    assert_eq!(version, oottracker::version()); // also checks oottracker-derive
    assert_eq!(version, oottracker_bizhawk::version());
    //assert_eq!(version, oottracker_csharp::version()); //TODO
    check_cli_version("oottracker-gui", &version).await;
    check_cli_version("oottracker-updater", &version).await;
    check_cli_version("oottracker-updater-bizhawk", &version).await;
    check_cli_version("oottracker-web", &version).await;
    version
}
