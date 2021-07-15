#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        fmt,
        io,
        path::PathBuf,
        process::Command,
        time::Duration,
    },
    derive_more::From,
    itertools::Itertools as _,
    structopt::StructOpt,
    tokio::{
        fs::File,
        io::AsyncWriteExt as _,
        time::sleep,
    },
    tokio_stream::StreamExt as _,
    oottracker::github::Repo,
};

#[cfg(target_arch = "x86")]
const PLATFORM_SUFFIX: &str = "-win32.exe";
#[cfg(target_arch = "x86_64")]
const PLATFORM_SUFFIX: &str = "-win64.exe";

#[derive(StructOpt)]
struct Args {
    path: PathBuf,
}

#[derive(From)]
enum Error {
    Io(io::Error),
    MissingAsset,
    NoReleases,
    Reqwest(reqwest::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::MissingAsset => write!(f, "release does not have a download for this platform"),
            Error::NoReleases => write!(f, "there are no released versions"),
            Error::Reqwest(e) => if let Some(url) = e.url() {
                write!(f, "HTTP error at {}: {}", url, e)
            } else {
                write!(f, "HTTP error: {}", e)
            },
        }
    }
}

#[wheel::main]
async fn main(args: Args) -> Result<(), Error> {
    println!("downloading update...");
    let client = reqwest::Client::builder()
        .user_agent(concat!("oottracker-updater/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let release = Repo::new("fenhl", "oottracker").latest_release(&client).await?.ok_or(Error::NoReleases)?;
    let (asset,) = release.assets.into_iter()
        .filter(|asset| asset.name.ends_with(PLATFORM_SUFFIX))
        .collect_tuple().ok_or(Error::MissingAsset)?;
    sleep(Duration::from_secs(1)).await; // to make sure the old version has exited
    let response = client.get(asset.browser_download_url).send().await?.error_for_status()?;
    println!("replacing app with new version...");
    {
        let mut data = response.bytes_stream();
        let mut exe_file = File::create(&args.path).await?;
        while let Some(chunk) = data.try_next().await? {
            exe_file.write_all(chunk.as_ref()).await?;
        }
    }
    println!("starting new version...");
    sleep(Duration::from_secs(1)).await; // to make sure the download is closed
    Command::new(args.path).spawn()?;
    Ok(())
}
