//! No Rust code here, this crate just stores the C# code for the BizHawk tool

use semver::Version;

pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version")
}
