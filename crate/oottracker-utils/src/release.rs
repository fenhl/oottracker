#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]

use {
    std::{
        cmp::Ordering::*,
        env,
        io::{
            self,
            Cursor,
        },
        path::Path,
        process::Output,
        time::Duration,
    },
    async_trait::async_trait,
    derive_more::From,
    dir_lock::DirLock,
    semver::{
        SemVerError,
        Version,
    },
    tempfile::NamedTempFile,
    tokio::{
        fs::File,
        prelude::*,
        process::Command,
    },
    zip::{
        ZipWriter,
        result::ZipError,
        write::FileOptions,
    },
    crate::github::{
        Release,
        Repo,
    },
};

mod github;

#[derive(Debug, From)]
enum Error {
    CommandExit(&'static str, Output),
    DirLock(dir_lock::Error),
    InvalidHeaderValue(reqwest::header::InvalidHeaderValue),
    Io(io::Error),
    MissingEnvar(&'static str),
    Reqwest(reqwest::Error),
    SameVersion,
    SemVer(SemVerError),
    VersionRegression,
    Zip(ZipError),
}

#[async_trait]
trait CommandOutputExt {
    async fn check(&mut self, name: &'static str) -> Result<Output, Error>;
}

#[async_trait]
impl CommandOutputExt for Command {
    async fn check(&mut self, name: &'static str) -> Result<Output, Error> {
        let output = self.output().await?;
        if output.status.success() {
            Ok(output)
        } else {
            Err(Error::CommandExit(name, output))
        }
    }
}

async fn release_client() -> Result<reqwest::Client, Error> {
    let mut headers = reqwest::header::HeaderMap::new();
    let mut token = String::default();
    File::open("assets/release-token").await?.read_to_string(&mut token).await?;
    headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!("token {}", token))?);
    headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static(concat!("oottracker-release/", env!("CARGO_PKG_VERSION"))));
    Ok(reqwest::Client::builder().default_headers(headers).timeout(Duration::from_secs(600)).build()?)
}

fn version() -> Version {
    //TODO make sure versions of all crates are equal
    Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version")
}

async fn setup() -> Result<(reqwest::Client, Repo), Error> {
    eprintln!("creating reqwest client");
    let client = release_client().await?;
    //TODO make sure working dir is clean and on default branch and up to date with remote and remote is up to date
    let repo = Repo::new("fenhl", "oottracker");
    eprintln!("checking version");
    if let Some(latest_release) = repo.latest_release(&client).await? {
        let remote_version = latest_release.version()?;
        match version().cmp(&remote_version) {
            Less => return Err(Error::VersionRegression),
            Equal => return Err(Error::SameVersion),
            Greater => {}
        }
    }
    eprintln!("waiting for Rust lock");
    let lock_dir = Path::new(&env::var_os("TEMP").ok_or(Error::MissingEnvar("TEMP"))?).join("syncbin-startup-rust.lock");
    let lock = DirLock::new(&lock_dir).await?;
    eprintln!("updating Rust for x86_64");
    Command::new("rustup").arg("update").arg("stable").check("rustup").await?;
    lock.drop_async().await?;
    Ok((client, repo))
}

async fn build_bizhawk(client: &reqwest::Client, repo: &Repo, release: &Release) -> Result<(), Error> {
    eprintln!("building oottracker-csharp");
    Command::new("cargo").arg("build").arg("--package=oottracker-csharp").check("cargo").await?; //TODO figure out why release builds crash at runtime, then reenable --release here
    eprintln!("building oottracker-bizhawk");
    Command::new("cargo").arg("build").arg("--package=oottracker-bizhawk").check("cargo").await?; //TODO figure out why release builds crash at runtime, then reenable --release here
    eprintln!("creating oottracker-bizhawk-win64.zip");
    let mut buf = Cursor::<Vec<_>>::default();
    {
        let mut zip = ZipWriter::new(&mut buf); //TODO replace with an async zip writer
        zip.start_file("README.txt", FileOptions::default())?;
        io::copy(&mut std::fs::File::open("crate/oottracker-bizhawk/assets/README.txt")?, &mut zip)?;
        zip.start_file("OotAutoTracker.dll", FileOptions::default())?;
        io::copy(&mut std::fs::File::open("crate/oottracker-bizhawk/OotAutoTracker/BizHawk/ExternalTools/OotAutoTracker.dll")?, &mut zip)?;
        zip.start_file("oottracker.dll", FileOptions::default())?;
        io::copy(&mut std::fs::File::open("crate/oottracker-bizhawk/OotAutoTracker/BizHawk/ExternalTools/oottracker.dll")?, &mut zip)?;
    }
    eprintln!("uploading oottracker-bizhawk-win64.zip");
    repo.release_attach(client, release, "oottracker-bizhawk-win64.zip", "application/zip", buf.into_inner()).await?;
    Ok(())
}

async fn build_gui(client: &reqwest::Client, repo: &Repo, release: &Release) -> Result<(), Error> {
    eprintln!("building oottracker-win64.exe");
    Command::new("cargo").arg("build").arg("--release").arg("--package=oottracker-gui").check("cargo").await?;
    eprintln!("uploading oottracker-win64.exe");
    repo.release_attach(client, release, "oottracker-win64.exe", "application/vnd.microsoft.portable-executable", {
        let mut f = File::open("target/release/oottracker-gui.exe").await?;
        let mut buf = Vec::default();
        f.read_to_end(&mut buf).await?;
        buf
    }).await?;
    Ok(())
}

async fn write_release_notes() -> Result<String, Error> {
    eprintln!("editing release notes");
    let mut release_notes_file = tempfile::Builder::new()
        .prefix("oottracker-release-notes")
        .suffix(".md")
        .tempfile()?;
    Command::new("C:\\Program Files\\Microsoft VS Code\\bin\\code.cmd").arg("--wait").arg(release_notes_file.path()).check("code").await?;
    let mut buf = String::default();
    <NamedTempFile as io::Read>::read_to_string(&mut release_notes_file, &mut buf)?;
    Ok(buf)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (setup_res, release_notes) = tokio::join!(
        setup(),
        write_release_notes(),
    );
    let (client, repo) = setup_res?;
    let release_notes = release_notes?;
    eprintln!("creating release");
    let release = repo.create_release(&client, format!("OoT Tracker {}", version()), format!("v{}", version()), release_notes).await?;
    let (build_bizhawk_res, build_gui_res) = tokio::join!(
        build_bizhawk(&client, &repo, &release),
        build_gui(&client, &repo, &release),
    );
    let () = build_bizhawk_res?;
    let () = build_gui_res?;
    eprintln!("publishing release");
    repo.publish_release(&client, release).await?;
    Ok(())
}
