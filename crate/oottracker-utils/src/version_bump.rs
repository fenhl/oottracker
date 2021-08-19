#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        fmt,
        num::ParseIntError,
    },
    derive_more::From,
    semver::Version,
    structopt::StructOpt,
    tokio::{
        fs,
        io,
    },
    toml_edit::TomlError,
    crate::version::version,
};

mod version;

#[derive(StructOpt)]
enum Args {
    Major,
    Minor,
    Patch,
    Exact {
        version: Version,
    },
}

#[derive(From)]
enum Error {
    Io(io::Error),
    MissingPackageEntry,
    PackageIsNotTable,
    ParseInt(ParseIntError),
    Plist(plist::Error),
    Toml(TomlError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::MissingPackageEntry => write!(f, "found Cargo manifest without “package” entry"),
            Error::PackageIsNotTable => write!(f, "“package” entry in Cargo manifest was not a table"),
            Error::ParseInt(e) => e.fmt(f),
            Error::Plist(e) => write!(f, "plist error: {}", e),
            Error::Toml(e) => e.fmt(f),
        }
    }
}

#[wheel::main]
async fn main(args: Args) -> Result<(), Error> {
    let version = match args {
        Args::Major => { let mut version = version().await; version.increment_major(); version }
        Args::Minor => { let mut version = version().await; version.increment_minor(); version }
        Args::Patch => { let mut version = version().await; version.increment_patch(); version }
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
        *manifest.as_table_mut().get_mut("package").ok_or(Error::MissingPackageEntry)?.as_table_mut().ok_or(Error::PackageIsNotTable)?.entry("version")
            = toml_edit::Item::Value(toml_edit::decorated(version.to_string().into(), " ", ""));
        fs::write(manifest_path, manifest.to_string().into_bytes()).await?;
    }
    Ok(())
}
