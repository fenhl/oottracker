#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        io,
        process::Output,
    },
    async_trait::async_trait,
    derive_more::From,
    ::tokio::process::Command,
};
#[cfg(target_os = "macos")] use tokio::fs;
#[cfg(windows)] use {
    std::{
        cmp::Ordering::*,
        env,
        fmt,
        io::Cursor,
        path::Path,
        process::Stdio,
        time::Duration,
    },
    dir_lock::DirLock,
    itertools::Itertools as _,
    semver::{
        SemVerError,
        Version,
    },
    serde::Deserialize,
    structopt::StructOpt,
    tempfile::NamedTempFile,
    ::tokio::{
        fs::File,
        io::AsyncReadExt as _,
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

#[cfg(windows)] mod github;

#[derive(Debug, From)]
enum Error {
    CommandExit(&'static str, Output),
    #[cfg(windows)]
    DirLock(dir_lock::Error),
    #[cfg(windows)]
    EmptyReleaseNotes,
    #[cfg(windows)]
    InvalidHeaderValue(reqwest::header::InvalidHeaderValue),
    Io(io::Error),
    #[cfg(windows)]
    MissingEnvar(&'static str),
    #[cfg(windows)]
    Reqwest(reqwest::Error),
    #[cfg(windows)]
    SameVersion,
    #[cfg(windows)]
    SemVer(SemVerError),
    #[cfg(windows)]
    VersionRegression,
    #[cfg(windows)]
    Zip(ZipError),
}

#[cfg(windows)]
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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

#[cfg(target_os = "macos")]
trait IoResultExt {
    fn exist_ok(self) -> Self;
}

#[cfg(target_os = "macos")]
impl IoResultExt for io::Result<()> {
    fn exist_ok(self) -> io::Result<()> {
        match self {
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
            _ => self,
        }
    }
}

#[cfg(windows)]
#[derive(StructOpt)]
struct Args {
    #[structopt(long = "web-dev-only")]
    web_dev_only: bool,
}

#[cfg(windows)]
async fn release_client() -> Result<reqwest::Client, Error> {
    let mut headers = reqwest::header::HeaderMap::new();
    let mut token = String::default();
    File::open("assets/release-token").await?.read_to_string(&mut token).await?;
    headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!("token {}", token))?);
    headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static(concat!("oottracker-release/", env!("CARGO_PKG_VERSION"))));
    Ok(reqwest::Client::builder().default_headers(headers).timeout(Duration::from_secs(600)).build()?)
}

#[cfg(windows)]
#[derive(Deserialize)]
struct Plist {
    #[serde(rename = "CFBundleShortVersionString")]
    bundle_short_version_string: Version,
}

#[cfg(windows)]
async fn version() -> Version {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).expect("failed to parse current version");
    assert_eq!(version, plist::from_file::<_, Plist>("assets/macos/OoT Tracker.app/Contents/Info.plist").expect("failed to read plist for version check").bundle_short_version_string);
    assert_eq!(version, ootr::version());
    assert_eq!(version, ootr_dynamic::version());
    assert_eq!(version, ootr_static::version()); // also checks ootr-static-derive
    assert_eq!(version, oottracker::version()); // also checks oottracker-derive
    assert_eq!(version, oottracker_bizhawk::version());
    //assert_eq!(version, oottracker_csharp::version()); //TODO
    let gui_output = String::from_utf8(Command::new("cargo").arg("run").arg("--package=oottracker-gui").arg("--").arg("--version").stdout(Stdio::piped()).output().await.expect("failed to run GUI with --version").stdout).expect("gui version output is invalid UTF-8");
    let (gui_name, gui_version) = gui_output.split(' ').collect_tuple().expect("no space in gui version output");
    assert_eq!(gui_name, "oottracker-gui");
    assert_eq!(version, gui_version.parse().expect("failed to parse GUI version"));
    version
}

#[cfg(windows)]
async fn setup() -> Result<(reqwest::Client, Repo), Error> {
    eprintln!("creating reqwest client");
    let client = release_client().await?;
    //TODO make sure working dir is clean and on default branch and up to date with remote and remote is up to date
    let repo = Repo::new("fenhl", "oottracker");
    eprintln!("checking version");
    if let Some(latest_release) = repo.latest_release(&client).await? {
        let remote_version = latest_release.version()?;
        match version().await.cmp(&remote_version) {
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

#[cfg(windows)]
async fn build_bizhawk(client: &reqwest::Client, repo: &Repo, release: &Release) -> Result<(), Error> {
    eprintln!("building oottracker-csharp");
    Command::new("cargo").arg("build").arg("--package=oottracker-csharp").check("cargo build --package=oottracker-csharp").await?; //TODO figure out why release builds crash at runtime, then reenable --release here
    eprintln!("building oottracker-bizhawk");
    Command::new("cargo").arg("build").arg("--package=oottracker-bizhawk").check("cargo build --package=oottracker-bizhawk").await?; //TODO figure out why release builds crash at runtime, then reenable --release here
    eprintln!("creating oottracker-bizhawk-win64.zip");
    let mut buf = Cursor::<Vec<_>>::default();
    {
        let mut zip = ZipWriter::new(&mut buf); //TODO replace with an async zip writer
        zip.start_file("README.txt", FileOptions::default())?;
        io::copy(&mut std::fs::File::open("crate/oottracker-bizhawk/assets/README.txt")?, &mut zip)?; //TODO auto-update BizHawk version
        zip.start_file("OotAutoTracker.dll", FileOptions::default())?;
        io::copy(&mut std::fs::File::open("crate/oottracker-bizhawk/OotAutoTracker/BizHawk/ExternalTools/OotAutoTracker.dll")?, &mut zip)?;
        zip.start_file("oottracker.dll", FileOptions::default())?;
        io::copy(&mut std::fs::File::open("crate/oottracker-bizhawk/OotAutoTracker/BizHawk/ExternalTools/oottracker.dll")?, &mut zip)?;
    }
    eprintln!("uploading oottracker-bizhawk-win64.zip");
    repo.release_attach(client, release, "oottracker-bizhawk-win64.zip", "application/zip", buf.into_inner()).await?;
    Ok(())
}

#[cfg(windows)]
async fn build_gui(client: &reqwest::Client, repo: &Repo, release: &Release) -> Result<(), Error> {
    eprintln!("building oottracker-win64.exe");
    Command::new("cargo").arg("build").arg("--release").arg("--package=oottracker-gui").check("cargo build --package=oottracker-gui").await?;
    eprintln!("uploading oottracker-win64.exe");
    repo.release_attach(client, release, "oottracker-win64.exe", "application/vnd.microsoft.portable-executable", {
        let mut f = File::open("target/release/oottracker-gui.exe").await?;
        let mut buf = Vec::default();
        f.read_to_end(&mut buf).await?;
        buf
    }).await?;
    Ok(())
}

#[cfg(windows)]
async fn build_macos(client: &reqwest::Client, repo: &Repo, release: &Release) -> Result<(), Error> {
    eprintln!("updating repo on bureflux");
    Command::new("ssh").arg("bureflux").arg("zsh").arg("-c").arg("'cd /opt/git/github.com/fenhl/oottracker/master && git pull --ff-only'").check("ssh").await?;
    eprintln!("connecting to bureflux");
    Command::new("ssh").arg("bureflux").arg("/opt/git/github.com/fenhl/oottracker/master/assets/release.sh").check("ssh").await?;
    eprintln!("downloading oottracker-mac-intel.dmg from bureflux");
    Command::new("scp").arg("bureflux:/opt/git/github.com/fenhl/oottracker/master/assets/oottracker-mac-intel.dmg").arg("assets/oottracker-mac-intel.dmg").check("scp").await?;
    eprintln!("uploading oottracker-mac-intel.dmg");
    repo.release_attach(client, release, "oottracker-mac-intel.dmg", "application/x-apple-diskimage", {
        let mut f = File::open("assets/oottracker-mac-intel.dmg").await?;
        let mut buf = Vec::default();
        f.read_to_end(&mut buf).await?;
        buf
    }).await?;
    Ok(())
}

#[cfg(windows)]
async fn build_web(dev: bool) -> Result<(), Error> {
    let www_path = if dev { "/var/www/oottracker-dev.fenhl.net" } else { "/var/www/oottracker.fenhl.net" };
    eprintln!("building for wasm");
    Command::new("cargo").arg("build").arg("--release").arg("--package=oottracker-gui").arg("--target=wasm32-unknown-unknown").check("cargo build --target=wasm32-unknown-unknown").await?;
    Command::new("wasm-bindgen").arg("target/wasm32-unknown-unknown/release/oottracker-gui.wasm").arg("--out-dir=assets/wasm").arg("--target=web").check("wasm-bindgen").await?;
    eprintln!("uploading web app");
    Command::new("scp").arg("-r").arg("assets/wasm/*").arg(format!("mercredi:{}", www_path)).check("scp").await?;
    Command::new("scp").arg("assets/icon.ico").arg(format!("mercredi:{}/favicon.ico", www_path)).check("scp").await?;
    Command::new("scp").arg("-r").arg("assets/xopar-*").arg(format!("mercredi:{}/assets", www_path)).check("scp").await?;
    Ok(())
}

#[cfg(windows)]
async fn write_release_notes() -> Result<String, Error> {
    eprintln!("editing release notes");
    let mut release_notes_file = tempfile::Builder::new()
        .prefix("oottracker-release-notes")
        .suffix(".md")
        .tempfile()?;
    Command::new("C:\\Program Files\\Microsoft VS Code\\bin\\code.cmd").arg("--wait").arg(release_notes_file.path()).check("code").await?;
    let mut buf = String::default();
    <NamedTempFile as io::Read>::read_to_string(&mut release_notes_file, &mut buf)?;
    if buf.is_empty() { return Err(Error::EmptyReleaseNotes) }
    Ok(buf)
}

#[cfg(target_os = "macos")]
#[tokio::main]
async fn main() -> Result<(), Error> {
    eprintln!("building oottracker-mac-intel.app");
    Command::new("cargo").arg("build").arg("--release").arg("--package=oottracker-gui").check("cargo").await?;
    fs::create_dir("assets/macos/OoT Tracker.app/Contents/MacOS").await.exist_ok()?;
    fs::copy("target/release/oottracker-gui", "assets/macos/OoT Tracker.app/Contents/MacOS/oottracker-gui").await?;
    eprintln!("packing oottracker-mac-intel.dmg");
    Command::new("hdiutil").arg("create").arg("assets/oottracker-mac-intel.dmg").arg("-volname").arg("OoT Tracker").arg("-srcfolder").arg("assets/macos").arg("-ov").check("hdiutil").await?;
    Ok(())
}

#[cfg(windows)]
#[wheel::main]
async fn main(Args { web_dev_only }: Args) -> Result<(), Error> {
    if web_dev_only {
        build_web(true).await?;
    } else {
        let (setup_res, release_notes) = tokio::join!(
            setup(),
            write_release_notes(),
        );
        let (client, repo) = setup_res?;
        let release_notes = release_notes?;
        eprintln!("creating release");
        let release = repo.create_release(&client, format!("OoT Tracker {}", version().await), format!("v{}", version().await), release_notes).await?;
        let (build_bizhawk_res, build_gui_res, build_macos_res, build_web_res) = tokio::join!(
            build_bizhawk(&client, &repo, &release),
            build_gui(&client, &repo, &release),
            build_macos(&client, &repo, &release),
            build_web(false),
        );
        let () = build_bizhawk_res?;
        let () = build_gui_res?;
        let () = build_macos_res?;
        let () = build_web_res?;
        eprintln!("publishing release");
        repo.publish_release(&client, release).await?;
    }
    Ok(())
}
