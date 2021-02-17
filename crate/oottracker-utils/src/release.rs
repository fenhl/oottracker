#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        fmt,
        io,
        process::ExitStatus,
    },
    async_trait::async_trait,
    derive_more::From,
    structopt::StructOpt,
    ::tokio::{
        fs,
        process::Command,
    },
};
#[cfg(windows)] use {
    std::{
        cmp::Ordering::*,
        env,
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
    tempfile::NamedTempFile,
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
    CommandExit(&'static str, ExitStatus),
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[async_trait]
trait CommandOutputExt {
    async fn check(&mut self, name: &'static str, verbose: bool) -> Result<ExitStatus, Error>;
}

#[async_trait]
impl CommandOutputExt for Command {
    async fn check(&mut self, name: &'static str, verbose: bool) -> Result<ExitStatus, Error> {
        let status = if verbose {
            self.status().await?
        } else {
            self.output().await?.status
        };
        if status.success() {
            Ok(status)
        } else {
            Err(Error::CommandExit(name, status))
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
async fn release_client() -> Result<reqwest::Client, Error> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!("token {}", fs::read_to_string("assets/release-token").await?))?);
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
async fn check_cli_version(package: &str, version: &Version) {
    let cli_output = String::from_utf8(Command::new("cargo").arg("run").arg(format!("--package={}", package)).arg("--").arg("--version").stdout(Stdio::piped()).output().await.expect("failed to run CLI with --version").stdout).expect("CLI version output is invalid UTF-8");
    let (cli_name, cli_version) = cli_output.split(' ').collect_tuple().expect("no space in CLI version output");
    assert_eq!(cli_name, package);
    assert_eq!(*version, cli_version.parse().expect("failed to parse CLI version"));
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
    check_cli_version("oottracker-gui", &version).await;
    check_cli_version("oottracker-web", &version).await;
    version
}

#[cfg(windows)]
async fn setup(verbose: bool) -> Result<(reqwest::Client, Repo), Error> {
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
    Command::new("rustup").arg("update").arg("stable").check("rustup", verbose).await?;
    lock.drop_async().await?;
    Ok((client, repo))
}

#[cfg(windows)]
async fn build_bizhawk(client: &reqwest::Client, repo: &Repo, release: &Release, verbose: bool) -> Result<(), Error> {
    eprintln!("building oottracker-csharp");
    Command::new("cargo").arg("build").arg("--package=oottracker-csharp").check("cargo build --package=oottracker-csharp", verbose).await?; //TODO figure out why release builds crash at runtime, then reenable --release here
    eprintln!("building oottracker-bizhawk");
    Command::new("cargo").arg("build").arg("--package=oottracker-bizhawk").check("cargo build --package=oottracker-bizhawk", verbose).await?; //TODO figure out why release builds crash at runtime, then reenable --release here
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
async fn build_gui(client: &reqwest::Client, repo: &Repo, release: &Release, verbose: bool) -> Result<(), Error> {
    eprintln!("building oottracker-win64.exe");
    Command::new("cargo").arg("build").arg("--release").arg("--package=oottracker-gui").check("cargo build --package=oottracker-gui", verbose).await?;
    eprintln!("uploading oottracker-win64.exe");
    repo.release_attach(client, release, "oottracker-win64.exe", "application/vnd.microsoft.portable-executable", fs::read("target/release/oottracker-gui.exe").await?).await?;
    Ok(())
}

#[cfg(windows)]
async fn build_macos(client: &reqwest::Client, repo: &Repo, release: &Release, verbose: bool) -> Result<(), Error> {
    eprintln!("updating repo on bureflux");
    Command::new("ssh").arg("bureflux").arg("zsh").arg("-c").arg("'cd /opt/git/github.com/fenhl/oottracker/master && git pull --ff-only'").check("ssh", verbose).await?;
    eprintln!("connecting to bureflux");
    Command::new("ssh").arg("bureflux").arg("/opt/git/github.com/fenhl/oottracker/master/assets/release.sh").arg(if verbose { "--verbose" } else { "" }).check("ssh", true).await?;
    eprintln!("downloading oottracker-mac.dmg from bureflux");
    Command::new("scp").arg("bureflux:/opt/git/github.com/fenhl/oottracker/master/assets/oottracker-mac.dmg").arg("assets/oottracker-mac.dmg").check("scp", verbose).await?;
    eprintln!("uploading oottracker-mac.dmg");
    repo.release_attach(client, release, "oottracker-mac.dmg", "application/x-apple-diskimage", fs::read("assets/oottracker-mac.dmg").await?).await?;
    Ok(())
}

#[cfg(windows)]
async fn build_web(verbose: bool) -> Result<(), Error> {
    Command::new("ssh").arg("mercredi").arg("sudo").arg("systemctl").arg("restart").arg("oottracker-web").check("ssh", verbose).await?;
    Ok(())
}

#[cfg(windows)]
async fn write_release_notes(args: &Args) -> Result<String, Error> {
    eprintln!("editing release notes");
    let mut release_notes_file = tempfile::Builder::new()
        .prefix("oottracker-release-notes")
        .suffix(".md")
        .tempfile()?;
    let mut cmd = Command::new(&args.release_notes_editor);
    if !args.no_wait {
        cmd.arg("--wait");
    }
    cmd.arg(release_notes_file.path()).check("code", args.verbose).await?;
    let mut buf = String::default();
    <NamedTempFile as io::Read>::read_to_string(&mut release_notes_file, &mut buf)?;
    if buf.is_empty() { return Err(Error::EmptyReleaseNotes) }
    Ok(buf)
}

#[derive(StructOpt)]
struct Args {
    #[cfg(windows)]
    /// Create the GitHub release as a draft
    #[structopt(long)]
    no_publish: bool,
    #[cfg(windows)]
    /// Don't pass `--wait` to the release notes editor
    #[structopt(short = "W", long)]
    no_wait: bool,
    #[cfg(windows)]
    /// the editor for the release notes
    #[structopt(short = "e", long, default_value = "C:\\Program Files\\Microsoft VS Code\\bin\\code.cmd")]
    release_notes_editor: String,
    /// Show output of build commands
    #[structopt(short, long)]
    verbose: bool,
}

#[cfg(target_os = "macos")]
#[wheel::main]
async fn main(args: Args) -> Result<(), Error> {
    eprintln!("building oottracker-mac.app for x86_64");
    Command::new("cargo").arg("build").arg("--release").arg("--target=x86_64-apple-darwin").arg("--package=oottracker-gui").check("cargo", args.verbose).await?;
    eprintln!("building oottracker-mac.app for aarch64");
    Command::new("cargo").arg("build").arg("--release").arg("--target=aarch64-apple-darwin").arg("--package=oottracker-gui").check("cargo", args.verbose).await?;
    eprintln!("creating Universal macOS binary");
    fs::create_dir("assets/macos/OoT Tracker.app/Contents/MacOS").await.exist_ok()?;
    Command::new("lipo").arg("-create").arg("target/aarch64-apple-darwin/release/oottracker-gui").arg("target/x86_64-apple-darwin/release/oottracker-gui").arg("-output").arg("assets/macos/OoT Tracker.app/Contents/MacOS/oottracker-gui").check("lipo", args.verbose).await?;
    eprintln!("packing oottracker-mac.dmg");
    Command::new("hdiutil").arg("create").arg("assets/oottracker-mac.dmg").arg("-volname").arg("OoT Tracker").arg("-srcfolder").arg("assets/macos").arg("-ov").check("hdiutil", args.verbose).await?;
    Ok(())
}

#[cfg(windows)]
#[wheel::main]
async fn main(args: Args) -> Result<(), Error> {
    let ((client, repo), release_notes) = if args.verbose {
        (
            setup(args.verbose).await?,
            write_release_notes(&args).await?,
        )
    } else {
        let (setup_res, release_notes) = tokio::join!(
            setup(args.verbose),
            write_release_notes(&args),
        );
        (setup_res?, release_notes?)
    };
    eprintln!("creating release");
    let release = repo.create_release(&client, format!("OoT Tracker {}", version().await), format!("v{}", version().await), release_notes).await?;
    if args.verbose {
        build_bizhawk(&client, &repo, &release, args.verbose).await?;
        build_gui(&client, &repo, &release, args.verbose).await?;
        build_macos(&client, &repo, &release, args.verbose).await?;
        build_web(args.verbose).await?;
    } else {
        let (build_bizhawk_res, build_gui_res, build_macos_res, build_web_res) = tokio::join!(
            build_bizhawk(&client, &repo, &release, args.verbose),
            build_gui(&client, &repo, &release, args.verbose),
            build_macos(&client, &repo, &release, args.verbose),
            build_web(args.verbose),
        );
        let () = build_bizhawk_res?;
        let () = build_gui_res?;
        let () = build_macos_res?;
        let () = build_web_res?;
    }
    if !args.no_publish {
        eprintln!("publishing release");
        repo.publish_release(&client, release).await?;
    }
    Ok(())
}
