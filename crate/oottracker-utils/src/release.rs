#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        fmt,
        process::ExitStatus,
    },
    async_trait::async_trait,
    derive_more::From,
    structopt::StructOpt,
    ::tokio::{
        fs,
        io,
        process::Command,
    },
};
#[cfg(windows)] use {
    std::{
        cmp::Ordering::*,
        env,
        io::{
            Cursor,
            Read as _,
            SeekFrom,
            Write as _,
        },
        iter,
        num::ParseIntError,
        path::Path,
        str::FromStr as _,
        time::Duration,
    },
    dir_lock::DirLock,
    graphql_client::{
        GraphQLQuery,
        Response,
    },
    itertools::Itertools as _,
    lazy_regex::regex_captures,
    semver::Version,
    tokio::{
        fs::File,
        io::{
            AsyncBufReadExt as _,
            AsyncSeekExt as _,
            BufReader,
        },
        sync::broadcast,
    },
    zip::{
        ZipWriter,
        result::ZipError,
        write::FileOptions,
    },
    oottracker::github::{
        Release,
        Repo,
    },
    crate::version::version,
};

#[cfg(windows)] mod version;

#[cfg(windows)] const MACOS_ADDR: &str = "192.168.178.63";

#[allow(dead_code)] // some fields are only used for Debug
#[derive(Debug, From)]
enum Error {
    #[cfg(windows)] BizHawkOutdated {
        latest: Version,
        local: Version,
    },
    #[cfg(windows)] BizHawkVersionRegression,
    #[cfg(windows)] BroadcastRecv(broadcast::error::RecvError),
    CommandExit(&'static str, ExitStatus),
    #[cfg(windows)] DirLock(dir_lock::Error),
    #[cfg(windows)] EmptyReleaseNotes,
    #[cfg(windows)] EmptyResponse,
    #[cfg(windows)] InvalidHeaderValue(reqwest::header::InvalidHeaderValue),
    Io(io::Error),
    #[cfg(windows)] MissingBizHawkRepo,
    #[cfg(windows)] MissingEnvar(&'static str),
    #[cfg(windows)] NoBizHawkReleases,
    #[cfg(windows)] ParseInt(ParseIntError),
    #[cfg(windows)] ProtocolVersionMismatch,
    #[cfg(windows)] ReleaseSend(broadcast::error::SendError<Release>),
    #[cfg(windows)] Reqwest(reqwest::Error),
    #[cfg(windows)] SameVersion,
    #[cfg(windows)] SemVer(semver::Error),
    #[cfg(windows)] Task(tokio::task::JoinError),
    #[cfg(windows)] UnnamedRelease,
    #[cfg(windows)] VersionRegression,
    #[cfg(windows)] Zip(ZipError),
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
    Ok(reqwest::Client::builder()
        .user_agent(concat!("oottracker/", env!("CARGO_PKG_VERSION")))
        .default_headers(headers)
        .timeout(Duration::from_secs(600))
        .http2_prior_knowledge()
        .use_rustls_tls()
        .https_only(true)
        .build()?)
}

#[cfg(windows)]
async fn setup(verbose: bool) -> Result<(reqwest::Client, Repo, Version), Error> {
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
    eprintln!("checking BizHawk version");
    let [major, minor, patch, _] = oottracker_bizhawk::bizhawk_version();
    let local_version = Version::new(major.into(), minor.into(), patch.into());
    let remote_version_string = client.post("https://api.github.com/graphql")
        .bearer_auth(include_str!("../../../assets/release-token"))
        .json(&BizHawkVersionQuery::build_query(biz_hawk_version_query::Variables {}))
        .send().await?
        .error_for_status()?
        .json::<Response<biz_hawk_version_query::ResponseData>>().await?
        .data.ok_or(Error::EmptyResponse)?
        .repository.ok_or(Error::MissingBizHawkRepo)?
        .latest_release.ok_or(Error::NoBizHawkReleases)?
        .name.ok_or(Error::UnnamedRelease)?;
    let (major, minor, patch) = remote_version_string.split('.').map(u64::from_str).chain(iter::repeat(Ok(0))).next_tuple().expect("iter::repeat produces an infinite iterator");
    let remote_version = Version::new(major?, minor?, patch?);
    match local_version.cmp(&remote_version) {
        Less => return Err(Error::BizHawkOutdated { local: local_version, latest: remote_version }),
        Equal => {}
        Greater => return Err(Error::BizHawkVersionRegression),
    }
    eprintln!("waiting for Rust lock");
    let lock_dir = Path::new(&env::var_os("TEMP").ok_or(Error::MissingEnvar("TEMP"))?).join("syncbin-startup-rust.lock");
    let lock = DirLock::new(&lock_dir).await?;
    eprintln!("updating Rust for x86_64");
    Command::new("rustup").arg("update").arg("stable").check("rustup", verbose).await?;
    lock.drop_async().await?;
    Ok((client, repo, local_version))
}

#[cfg(windows)]
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../assets/graphql/github-schema.graphql",
    query_path = "../../assets/graphql/github-bizhawk-version.graphql",
)]
struct BizHawkVersionQuery;

#[cfg(windows)]
async fn build_bizhawk(client: &reqwest::Client, repo: &Repo, mut release_rx: broadcast::Receiver<Release>, verbose: bool, version: Version) -> Result<(), Error> {
    eprintln!("building oottracker-updater-bizhawk.exe");
    Command::new("cargo").arg("build").arg("--release").arg("--target=x86_64-pc-windows-msvc").arg("--package=oottracker-updater-bizhawk").check("cargo build --package=oottracker-updater-bizhawk", verbose).await?;
    eprintln!("building oottracker-csharp");
    Command::new("cargo").arg("build").arg("--release").arg("--package=oottracker-csharp").check("cargo build --package=oottracker-csharp", verbose).await?;
    eprintln!("building oottracker-bizhawk");
    Command::new("cargo").arg("build").arg("--release").arg("--package=oottracker-bizhawk").check("cargo build --package=oottracker-bizhawk", verbose).await?;
    eprintln!("creating oottracker-bizhawk-win64.zip");
    let mut buf = Cursor::<Vec<_>>::default();
    {
        let mut zip = ZipWriter::new(&mut buf); //TODO replace with an async zip writer
        zip.start_file("README.txt", FileOptions::default())?;
        write!(&mut zip, include_str!("../../../assets/bizhawk-readme.txt"), version)?;
        zip.start_file("OotAutoTracker.dll", FileOptions::default())?;
        std::io::copy(&mut std::fs::File::open("crate/oottracker-bizhawk/OotAutoTracker/BizHawk/ExternalTools/OotAutoTracker.dll")?, &mut zip)?;
        zip.start_file("oottracker.dll", FileOptions::default())?;
        std::io::copy(&mut std::fs::File::open("crate/oottracker-bizhawk/OotAutoTracker/BizHawk/ExternalTools/oottracker.dll")?, &mut zip)?;
    }
    eprintln!("uploading oottracker-bizhawk-win64.zip");
    repo.release_attach(client, &release_rx.recv().await?, "oottracker-bizhawk-win64.zip", "application/zip", buf.into_inner()).await?;
    eprintln!("BizHawk build done");
    Ok(())
}

#[cfg(windows)]
async fn build_gui(client: &reqwest::Client, repo: &Repo, mut release_rx: broadcast::Receiver<Release>, verbose: bool) -> Result<(), Error> {
    eprintln!("building oottracker-updater.exe");
    Command::new("cargo").arg("build").arg("--release").arg("--target=x86_64-pc-windows-msvc").arg("--package=oottracker-updater").check("cargo build --package=oottracker-updater", verbose).await?;
    eprintln!("building oottracker-win64.exe");
    Command::new("cargo").arg("build").arg("--release").arg("--package=oottracker-gui").check("cargo build --package=oottracker-gui", verbose).await?;
    eprintln!("uploading oottracker-win64.exe");
    repo.release_attach(client, &release_rx.recv().await?, "oottracker-win64.exe", "application/vnd.microsoft.portable-executable", fs::read("target/release/oottracker-gui.exe").await?).await?;
    eprintln!("Windows GUI build done");
    Ok(())
}

#[cfg(windows)]
async fn build_macos(client: &reqwest::Client, repo: &Repo, mut release_rx: broadcast::Receiver<Release>, verbose: bool) -> Result<(), Error> {
    eprintln!("updating repo on Mac");
    Command::new("ssh").arg(MACOS_ADDR).arg("cd /opt/git/github.com/fenhl/oottracker/master && git pull --ff-only").check("ssh", verbose).await?;
    eprintln!("connecting to Mac");
    Command::new("ssh").arg(MACOS_ADDR).arg("/opt/git/github.com/fenhl/oottracker/master/assets/release.sh").arg(if verbose { "--verbose" } else { "" }).check("ssh", true).await?; //TODO convert newlines ro \r\n
    eprintln!("downloading oottracker-mac.dmg from Mac");
    Command::new("scp").arg(format!("{}:/opt/git/github.com/fenhl/oottracker/master/assets/oottracker-mac.dmg", MACOS_ADDR)).arg("assets/oottracker-mac.dmg").check("scp", verbose).await?;
    eprintln!("uploading oottracker-mac.dmg");
    repo.release_attach(client, &release_rx.recv().await?, "oottracker-mac.dmg", "application/x-apple-diskimage", fs::read("assets/oottracker-mac.dmg").await?).await?;
    eprintln!("macOS build done");
    Ok(())
}

#[cfg(windows)]
async fn build_pj64(client: &reqwest::Client, repo: &Repo, mut release_rx: broadcast::Receiver<Release>) -> Result<(), Error> {
    eprintln!("compiling oottracker-pj64.js");
    let mut buf = Vec::default();
    writeln!(&mut buf, "const TCP_PORT = {};", oottracker::proto::TCP_PORT)?;
    writeln!(&mut buf, "const SAVE_ADDR = {};", oottracker::save::ADDR)?;
    writeln!(&mut buf, "const SAVE_SIZE = {};", oottracker::save::SIZE)?;
    writeln!(&mut buf, "const RAM_RANGES = [{}];", oottracker::ram::RANGES.iter()
        .copied()
        .tuples()
        .map(|(start, len)| format!("[{}, {}]", start, len))
        .join(", ")
    )?;
    let mut base = BufReader::new(File::open("assets/oottracker-pj64-base.js").await?).lines();
    while let Some(line) = base.next_line().await? {
        if let Some((_, version)) = regex_captures!("^const VERSION = ([0-9]+);", &line) {
            if version.parse::<u8>()? != oottracker::proto::VERSION {
                return Err(Error::ProtocolVersionMismatch)
            }
            break
        }
    }
    let mut base = base.into_inner();
    base.seek(SeekFrom::Start(0)).await?;
    io::copy(&mut base, &mut buf).await?;
    eprintln!("uploading oottracker-pj64.js");
    repo.release_attach(client, &release_rx.recv().await?, "oottracker-pj64.js", "text/javascript", buf).await?;
    eprintln!("Project64 build done");
    Ok(())
}

#[cfg(windows)]
async fn build_web(verbose: bool) -> Result<(), Error> {
    eprintln!("updating repo on mercredi");
    Command::new("ssh").arg("mercredi").arg("cd /opt/git/github.com/fenhl/oottracker/master && git pull --ff-only").check("ssh", verbose).await?;
    eprintln!("building oottracker.fenhl.net");
    Command::new("ssh").arg("mercredi").arg(concat!("env -C /opt/git/github.com/fenhl/oottracker/master ", include_str!("../../../assets/web/env.txt"), " cargo build --release --package=oottracker-web")).check("ssh", verbose).await?;
    eprintln!("restarting oottracker.fenhl.net");
    Command::new("ssh").arg("mercredi").arg("sudo systemctl restart oottracker-web").check("ssh", verbose).await?;
    eprintln!("web build done");
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
    release_notes_file.read_to_string(&mut buf)?;
    if buf.is_empty() { return Err(Error::EmptyReleaseNotes) }
    Ok(buf)
}

#[derive(Clone, StructOpt)]
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
    Command::new("cargo").arg("build").arg("--release").arg("--target=x86_64-apple-darwin").arg("--package=oottracker-gui").env("MACOSX_DEPLOYMENT_TARGET", "10.9").check("cargo", args.verbose).await?;
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
    let (client, repo, bizhawk_version) = setup(args.verbose).await?; // don't show release notes editor if version check could still fail
    let (release_tx, release_rx_bizhawk) = broadcast::channel(1);
    let release_rx_gui = release_tx.subscribe();
    let release_rx_macos = release_tx.subscribe();
    let release_rx_pj64 = release_tx.subscribe();
    let create_release_args = args.clone();
    let create_release_client = client.clone();
    let create_release_repo = repo.clone();
    let create_release = tokio::spawn(async move {
        let release_notes = write_release_notes(&create_release_args).await?;
        eprintln!("creating release");
        let release = create_release_repo.create_release(&create_release_client, format!("OoT Tracker {}", version().await), format!("v{}", version().await), release_notes).await?;
        release_tx.send(release.clone())?;
        Ok::<_, Error>(release)
    });

    macro_rules! with_metavariable {
        ($metavariable:tt, $($token:tt)*) => { $($token)* };
    }

    macro_rules! build_tasks {
        ($($task:expr,)*) => {
            if args.verbose {
                $($task.await?;)*
            } else {
                let ($(with_metavariable!($task, ())),*) = tokio::try_join!($($task),*)?;
            }
        };
    }

    build_tasks![
        build_bizhawk(&client, &repo, release_rx_bizhawk, args.verbose, bizhawk_version),
        build_gui(&client, &repo, release_rx_gui, args.verbose),
        build_macos(&client, &repo, release_rx_macos, args.verbose),
        build_pj64(&client, &repo, release_rx_pj64),
        build_web(args.verbose),
    ];
    let release = create_release.await??;
    if !args.no_publish {
        eprintln!("publishing release");
        repo.publish_release(&client, release).await?;
    }
    Ok(())
}
