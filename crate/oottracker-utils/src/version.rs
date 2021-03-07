use {
    std::process::Stdio,
    itertools::Itertools as _,
    semver::Version,
    serde::Deserialize,
    tokio::process::Command,
};

#[derive(Deserialize)]
struct Plist {
    #[serde(rename = "CFBundleShortVersionString")]
    bundle_short_version_string: Version,
}

pub async fn check_cli_version(package: &str, version: &Version) {
    let cli_output = String::from_utf8(Command::new("cargo").arg("run").arg(format!("--package={}", package)).arg("--").arg("--version").stdout(Stdio::piped()).output().await.expect("failed to run CLI with --version").stdout).expect("CLI version output is invalid UTF-8");
    let (cli_name, cli_version) = cli_output.split(' ').collect_tuple().expect("no space in CLI version output");
    assert_eq!(cli_name, package);
    assert_eq!(*version, cli_version.parse().expect("failed to parse CLI version"));
}

pub async fn version() -> Version {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version");
    assert_eq!(version, plist::from_file::<_, Plist>("assets/macos/OoT Tracker.app/Contents/Info.plist").expect("failed to read plist for version check").bundle_short_version_string);
    assert_eq!(version, ootr::version());
    assert_eq!(version, ootr_dynamic::version());
    assert_eq!(version, ootr_static::version()); // also checks ootr-static-derive
    assert_eq!(version, oottracker::version()); // also checks oottracker-derive
    assert_eq!(version, oottracker_bizhawk::version());
    //assert_eq!(version, oottracker_csharp::version()); //TODO
    check_cli_version("oottracker-gui", &version).await;
    check_cli_version("oottracker-web", &version).await;
    version
}
