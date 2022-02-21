#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::num::ParseIntError,
    semver::{
        BuildMetadata,
        Prerelease,
        Version,
    },
    tokio::{
        fs,
        io,
    },
    toml_edit::TomlError,
    oottracker_utils::version,
};

#[derive(clap::Parser)]
#[clap(version)]
enum Args {
    Major,
    Minor,
    Patch,
    Exact {
        version: Version,
    },
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)] Io(#[from] io::Error),
    #[error(transparent)] ParseInt(#[from] ParseIntError),
    #[error(transparent)] Plist(#[from] plist::Error),
    #[error(transparent)] Toml(#[from] TomlError),
    #[error("found Cargo manifest without “package” entry")]
    MissingPackageEntry,
    #[error("found “package” entry in Cargo manifest without “version” entry")]
    MissingVersionEntry,
    #[error("“package” entry in Cargo manifest was not a table")]
    PackageIsNotTable,
}

//FROM https://github.com/dtolnay/semver/issues/243#issuecomment-854337640
fn increment_patch(v: &mut Version) {
    v.patch += 1;
    v.pre = Prerelease::EMPTY;
    v.build = BuildMetadata::EMPTY;
}

fn increment_minor(v: &mut Version) {
    v.minor += 1;
    v.patch = 0;
    v.pre = Prerelease::EMPTY;
    v.build = BuildMetadata::EMPTY;
}

fn increment_major(v: &mut Version) {
    v.major += 1;
    v.minor = 0;
    v.patch = 0;
    v.pre = Prerelease::EMPTY;
    v.build = BuildMetadata::EMPTY;
}

#[wheel::main]
async fn main(args: Args) -> Result<(), Error> {
    let version = match args {
        Args::Major => { let mut version = version::version().await; increment_major(&mut version); version }
        Args::Minor => { let mut version = version::version().await; increment_minor(&mut version); version }
        Args::Patch => { let mut version = version::version().await; increment_patch(&mut version); version }
        Args::Exact { version } => version,
    };
    println!("new version: {}", version);
    let mut info_plist = plist::from_file::<_, version::Plist>(version::INFO_PLIST_PATH)?;
    info_plist.bundle_version = (info_plist.bundle_version.parse::<u64>()? + 1).to_string();
    info_plist.bundle_short_version_string = version.clone();
    plist::to_file_xml(version::INFO_PLIST_PATH, &info_plist)?;
    let mut crates = fs::read_dir("crate").await?;
    while let Some(entry) = crates.next_entry().await? {
        let manifest_path = entry.path().join("Cargo.toml");
        let mut manifest = fs::read_to_string(&manifest_path).await?.parse::<toml_edit::Document>()?;
        *manifest.as_table_mut().get_mut("package").ok_or(Error::MissingPackageEntry)?.as_table_mut().ok_or(Error::PackageIsNotTable)?.get_mut("version").ok_or(Error::MissingVersionEntry)?
            = toml_edit::Item::Value(toml_edit::Value::from(version.to_string()).decorated(" ", ""));
        fs::write(manifest_path, manifest.to_string().into_bytes()).await?;
    }
    Ok(())
}
