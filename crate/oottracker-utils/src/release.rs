#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        env,
        process::Stdio,
    },
    async_proto::Protocol,
    dir_lock::DirLock,
    gres::Percent,
    thiserror::Error,
    ::tokio::{
        fs,
        io,
        process::Command,
    },
    wheel::traits::AsyncCommandOutputExt as _,
};
#[cfg(windows)] use {
    std::{
        cmp::Ordering::*,
        ffi::OsString,
        fmt,
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
        sync::Arc,
        time::Duration,
    },
    async_proto::ReadError,
    async_trait::async_trait,
    graphql_client::{
        GraphQLQuery,
        Response,
    },
    gres::{
        Progress,
        Task,
        cli::Cli,
    },
    itertools::Itertools as _,
    lazy_regex::regex_captures,
    semver::Version,
    tempfile::NamedTempFile,
    tokio::{
        fs::File,
        io::{
            AsyncBufReadExt as _,
            AsyncSeekExt as _,
            BufReader,
        },
        process::{
            Child,
            ChildStdout,
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
#[cfg(target_os = "macos")] use {
    directories::BaseDirs,
    git2::{
        BranchType,
        Repository,
        ResetType,
    },
    tokio::io::stdout,
    wheel::traits::IoResultExt as _,
};

#[cfg(windows)] mod version;

#[cfg(windows)] const MACOS_ADDR: &str = "192.168.178.63";

#[derive(Debug, Error)]
enum Error {
    #[cfg(windows)] #[error(transparent)] BroadcastRecv(#[from] broadcast::error::RecvError),
    #[error(transparent)] DirLock(#[from] dir_lock::Error),
    #[cfg(target_os = "macos")] #[error(transparent)] Env(#[from] env::VarError),
    #[cfg(target_os = "macos")] #[error(transparent)] Git(#[from] git2::Error),
    #[cfg(windows)] #[error(transparent)] InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error(transparent)] Io(#[from] io::Error),
    #[cfg(windows)] #[error(transparent)] ParseInt(#[from] ParseIntError),
    #[cfg(windows)] #[error(transparent)] Read(#[from] ReadError),
    #[cfg(windows)] #[error(transparent)] ReleaseSend(#[from] broadcast::error::SendError<Release>),
    #[cfg(windows)] #[error(transparent)] Reqwest(#[from] reqwest::Error),
    #[cfg(windows)] #[error(transparent)] SemVer(#[from] semver::Error),
    #[cfg(windows)] #[error(transparent)] Task(#[from] tokio::task::JoinError),
    #[error(transparent)] Wheel(#[from] wheel::Error),
    #[cfg(target_os = "macos")] #[error(transparent)] Write(#[from] async_proto::WriteError),
    #[cfg(windows)] #[error(transparent)] Zip(#[from] ZipError),
    #[cfg(windows)]
    #[error("BizHawk is outdated ({local} installed, {latest} available)")]
    BizHawkOutdated {
        latest: Version,
        local: Version,
    },
    #[cfg(windows)]
    #[error("locally installed BizHawk is newer than latest release")]
    BizHawkVersionRegression,
    #[cfg(windows)]
    #[error("aborting due to empty release notes")]
    EmptyReleaseNotes,
    #[cfg(windows)]
    #[error("no info returned in BizHawk version query response")]
    EmptyBizHawkVersionResponse,
    #[cfg(windows)]
    #[error("no BizHawk repo info returned")]
    MissingBizHawkRepo,
    #[cfg(windows)]
    #[error("missing environment variable: {0}")]
    MissingEnvar(&'static str),
    #[cfg(windows)]
    #[error("no releases in BizHawk GitHub repo")]
    NoBizHawkReleases,
    #[cfg(windows)]
    #[error("Project64 plugin uses the wrong protocol version")]
    ProtocolVersionMismatch,
    #[cfg(windows)]
    #[error("there is already a release with this version number")]
    SameVersion,
    #[cfg(windows)]
    #[error("the latest GitHub release has no name")]
    UnnamedRelease,
    #[cfg(windows)]
    #[error("the latest GitHub release has a newer version than the local crate version")]
    VersionRegression,
}

#[cfg(windows)]
enum Setup {
    CreateReqwestClient,
    CheckVersion(reqwest::Client),
    CheckBizHawkVersion(reqwest::Client, Repo),
    LockRust(reqwest::Client, Repo, Version),
    UpdateRust(reqwest::Client, Repo, Version, DirLock),
}

#[cfg(windows)]
impl Default for Setup {
    fn default() -> Self {
        Self::CreateReqwestClient
    }
}

#[cfg(windows)]
impl fmt::Display for Setup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateReqwestClient => write!(f, "creating reqwest client"),
            Self::CheckVersion(..) => write!(f, "checking version"),
            Self::CheckBizHawkVersion(..) => write!(f, "checking BizHawk version"),
            Self::LockRust(..) => write!(f, "waiting for Rust lock"),
            Self::UpdateRust(..) => write!(f, "updating Rust for x86_64"),
        }
    }
}

#[cfg(windows)]
impl Progress for Setup {
    fn progress(&self) -> Percent {
        Percent::fraction(match self {
            Self::CreateReqwestClient => 0,
            Self::CheckVersion(..) => 1,
            Self::CheckBizHawkVersion(..) => 2,
            Self::LockRust(..) => 3,
            Self::UpdateRust(..) => 4,
        }, 5)
    }
}

#[cfg(windows)]
#[async_trait]
impl Task<Result<(reqwest::Client, Repo, Version), Error>> for Setup {
    async fn run(self) -> Result<Result<(reqwest::Client, Repo, Version), Error>, Self> {
        match self {
            Self::CreateReqwestClient => gres::transpose(async move {
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!("token {}", fs::read_to_string("assets/release-token").await?))?);
                headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static(concat!("oottracker-release/", env!("CARGO_PKG_VERSION"))));
                let client = reqwest::Client::builder()
                    .user_agent(concat!("oottracker/", env!("CARGO_PKG_VERSION")))
                    .default_headers(headers)
                    .timeout(Duration::from_secs(600))
                    .http2_prior_knowledge()
                    .use_rustls_tls()
                    .https_only(true)
                    .build()?;
                Ok(Err(Self::CheckVersion(client)))
            }).await,
            Self::CheckVersion(client) => gres::transpose(async move {
                //TODO make sure working dir is clean and on default branch and up to date with remote and remote is up to date
                let repo = Repo::new("fenhl", "oottracker");
                if let Some(latest_release) = repo.latest_release(&client).await? {
                    let remote_version = latest_release.version()?;
                    match version().await.cmp(&remote_version) {
                        Less => return Err(Error::VersionRegression),
                        Equal => return Err(Error::SameVersion),
                        Greater => {}
                    }
                }
                Ok(Err(Self::CheckBizHawkVersion(client, repo)))
            }).await,
            Self::CheckBizHawkVersion(client, repo) => gres::transpose(async move {
                let [major, minor, patch, _] = oottracker_bizhawk::bizhawk_version();
                let local_version = Version::new(major.into(), minor.into(), patch.into());
                let remote_version_string = client.post("https://api.github.com/graphql")
                    .bearer_auth(include_str!("../../../assets/release-token"))
                    .json(&BizHawkVersionQuery::build_query(biz_hawk_version_query::Variables {}))
                    .send().await?
                    .error_for_status()?
                    .json::<Response<biz_hawk_version_query::ResponseData>>().await?
                    .data.ok_or(Error::EmptyBizHawkVersionResponse)?
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
                Ok(Err(Self::LockRust(client, repo, local_version)))
            }).await,
            Self::LockRust(client, repo, local_version) => gres::transpose(async move {
                let lock_dir = Path::new(&env::var_os("TEMP").ok_or(Error::MissingEnvar("TEMP"))?).join("syncbin-startup-rust.lock");
                let lock = DirLock::new(&lock_dir).await?;
                Ok(Err(Self::UpdateRust(client, repo, local_version, lock))) //TODO update rustup first?
            }).await,
            Self::UpdateRust(client, repo, local_version, lock) => gres::transpose(async move {
                Command::new("rustup").arg("update").arg("stable").check("rustup").await?;
                lock.drop_async().await?;
                Ok(Ok((client, repo, local_version)))
            }).await,
        }
    }
}

#[cfg(windows)]
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../assets/graphql/github-schema.graphql",
    query_path = "../../assets/graphql/github-bizhawk-version.graphql",
)]
struct BizHawkVersionQuery;

#[cfg(windows)]
enum BuildBizHawk {
    Updater(reqwest::Client, Repo, broadcast::Receiver<Release>, Version),
    CSharp(reqwest::Client, Repo, broadcast::Receiver<Release>, Version),
    BizHawk(reqwest::Client, Repo, broadcast::Receiver<Release>, Version),
    Zip(reqwest::Client, Repo, broadcast::Receiver<Release>, Version),
    WaitRelease(reqwest::Client, Repo, broadcast::Receiver<Release>, Vec<u8>),
    Upload(reqwest::Client, Repo, Release, Vec<u8>),
}

#[cfg(windows)]
impl BuildBizHawk {
    fn new(client: reqwest::Client, repo: Repo, release_rx: broadcast::Receiver<Release>, version: Version) -> Self {
        Self::Updater(client, repo, release_rx, version)
    }
}

#[cfg(windows)]
impl fmt::Display for BuildBizHawk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Updater(..) => write!(f, "building oottracker-updater-bizhawk.exe"),
            Self::CSharp(..) => write!(f, "building oottracker-csharp"),
            Self::BizHawk(..) => write!(f, "building oottracker-bizhawk"),
            Self::Zip(..) => write!(f, "creating oottracker-bizhawk-win64.zip"),
            Self::WaitRelease(..) => write!(f, "waiting for GitHub release to be created"),
            Self::Upload(..) => write!(f, "uploading oottracker-bizhawk-win64.zip"),
        }
    }
}

#[cfg(windows)]
impl Progress for BuildBizHawk {
    fn progress(&self) -> Percent {
        Percent::fraction(match self {
            Self::Updater(..) => 0,
            Self::CSharp(..) => 1,
            Self::BizHawk(..) => 2,
            Self::Zip(..) => 3,
            Self::WaitRelease(..) => 4,
            Self::Upload(..) => 5,
        }, 6)
    }
}

#[cfg(windows)]
#[async_trait]
impl Task<Result<(), Error>> for BuildBizHawk {
    async fn run(self) -> Result<Result<(), Error>, Self> {
        match self {
            Self::Updater(client, repo, release_rx, version) => gres::transpose(async move {
                Command::new("cargo").arg("build").arg("--release").arg("--target=x86_64-pc-windows-msvc").arg("--package=oottracker-updater-bizhawk").check("cargo build --package=oottracker-updater-bizhawk").await?;
                Ok(Err(Self::CSharp(client, repo, release_rx, version)))
            }).await,
            Self::CSharp(client, repo, release_rx, version) => gres::transpose(async move {
                Command::new("cargo").arg("build").arg("--release").arg("--package=oottracker-csharp").check("cargo build --package=oottracker-csharp").await?;
                Ok(Err(Self::BizHawk(client, repo, release_rx, version)))
            }).await,
            Self::BizHawk(client, repo, release_rx, version) => gres::transpose(async move {
                Command::new("cargo").arg("build").arg("--release").arg("--package=oottracker-bizhawk").check("cargo build --package=oottracker-bizhawk").await?;
                Ok(Err(Self::Zip(client, repo, release_rx, version)))
            }).await,
            Self::Zip(client, repo, release_rx, version) => gres::transpose(async move {
                let zip_data = tokio::task::spawn_blocking(move || {
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
                    Ok::<_, Error>(buf.into_inner())
                }).await??;
                Ok(Err(Self::WaitRelease(client, repo, release_rx, zip_data)))
            }).await,
            Self::WaitRelease(client, repo, mut release_rx, zip_data) => gres::transpose(async move {
                let release = release_rx.recv().await?;
                Ok(Err(Self::Upload(client, repo, release, zip_data)))
            }).await,
            Self::Upload(client, repo, release, zip_data) => gres::transpose(async move {
                repo.release_attach(&client, &release, "oottracker-bizhawk-win64.zip", "application/zip", zip_data).await?;
                Ok(Ok(()))
            }).await,
        }
    }
}

#[cfg(windows)]
enum BuildGui {
    Updater(reqwest::Client, Repo, broadcast::Receiver<Release>),
    X64(reqwest::Client, Repo, broadcast::Receiver<Release>),
    ReadX64(reqwest::Client, Repo, broadcast::Receiver<Release>),
    WaitRelease(reqwest::Client, Repo, broadcast::Receiver<Release>, Vec<u8>),
    Upload(reqwest::Client, Repo, Release, Vec<u8>),
}

#[cfg(windows)]
impl BuildGui {
    fn new(client: reqwest::Client, repo: Repo, release_rx: broadcast::Receiver<Release>) -> Self {
        Self::Updater(client, repo, release_rx)
    }
}

#[cfg(windows)]
impl fmt::Display for BuildGui {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Updater(..) => write!(f, "building oottracker-updater.exe"),
            Self::X64(..) => write!(f, "building oottracker-win64.exe"),
            Self::ReadX64(..) => write!(f, "reading oottracker-win64.exe"),
            Self::WaitRelease(..) => write!(f, "waiting for GitHub release to be created"),
            Self::Upload(..) => write!(f, "uploading oottracker-win64.exe"),
        }
    }
}

#[cfg(windows)]
impl Progress for BuildGui {
    fn progress(&self) -> Percent {
        Percent::fraction(match self {
            Self::Updater(..) => 0,
            Self::X64(..) => 1,
            Self::ReadX64(..) => 2,
            Self::WaitRelease(..) => 3,
            Self::Upload(..) => 4,
        }, 5)
    }
}

#[cfg(windows)]
#[async_trait]
impl Task<Result<(), Error>> for BuildGui {
    async fn run(self) -> Result<Result<(), Error>, Self> {
        match self {
            Self::Updater(client, repo, release_rx) => gres::transpose(async move {
                Command::new("cargo").arg("build").arg("--release").arg("--target=x86_64-pc-windows-msvc").arg("--package=oottracker-updater").check("cargo build --package=oottracker-updater").await?;
                Ok(Err(Self::X64(client, repo, release_rx)))
            }).await,
            Self::X64(client, repo, release_rx) => gres::transpose(async move {
                Command::new("cargo").arg("build").arg("--release").arg("--package=oottracker-gui").check("cargo build --package=oottracker-gui").await?;
                Ok(Err(Self::ReadX64(client, repo, release_rx)))
            }).await,
            Self::ReadX64(client, repo, release_rx) => gres::transpose(async move {
                let x64_data = fs::read("target/release/oottracker-gui.exe").await?;
                Ok(Err(Self::WaitRelease(client, repo, release_rx, x64_data)))
            }).await,
            Self::WaitRelease(client, repo, mut release_rx, x64_data) => gres::transpose(async move {
                let release = release_rx.recv().await?;
                Ok(Err(Self::Upload(client, repo, release, x64_data)))
            }).await,
            Self::Upload(client, repo, release, x64_data) => gres::transpose(async move {
                repo.release_attach(&client, &release, "oottracker-win64.exe", "application/vnd.microsoft.portable-executable", x64_data).await?;
                Ok(Ok(()))
            }).await,
        }
    }
}

#[derive(Protocol)]
enum MacMessage {
    Progress {
        label: String,
        percent: Percent,
    },
}

#[cfg(windows)]
enum BuildMacOs {
    Connect(reqwest::Client, Repo, broadcast::Receiver<Release>),
    Remote(String, Percent, reqwest::Client, Repo, broadcast::Receiver<Release>, Child, ChildStdout),
    Disconnect(reqwest::Client, Repo, broadcast::Receiver<Release>, Child),
    Download(reqwest::Client, Repo, broadcast::Receiver<Release>),
    ReadDmg(reqwest::Client, Repo, broadcast::Receiver<Release>),
    WaitRelease(reqwest::Client, Repo, broadcast::Receiver<Release>, Vec<u8>),
    Upload(reqwest::Client, Repo, Release, Vec<u8>),
}

#[cfg(windows)]
impl BuildMacOs {
    fn new(client: reqwest::Client, repo: Repo, release_rx: broadcast::Receiver<Release>) -> Self {
        Self::Connect(client, repo, release_rx)
    }
}

#[cfg(windows)]
impl fmt::Display for BuildMacOs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(..) => write!(f, "connecting to Mac"),
            Self::Remote(msg, ..) => msg.fmt(f),
            Self::Disconnect(..) => write!(f, "disconnecting from Mac"),
            Self::Download(..) => write!(f, "downloading oottracker-mac.dmg from Mac"),
            Self::ReadDmg(..) => write!(f, "reading oottracker-mac.dmg"),
            Self::WaitRelease(..) => write!(f, "waiting for GitHub release to be created"),
            Self::Upload(..) => write!(f, "uploading oottracker-mac.dmg"),
        }
    }
}

#[cfg(windows)]
impl Progress for BuildMacOs {
    fn progress(&self) -> Percent {
        Percent::new(match self {
            Self::Connect(..) => 0,
            Self::Remote(_, percent, ..) => 5 + u8::from(percent) / 2,
            Self::Disconnect(..) => 60,
            Self::Download(..) => 65,
            Self::ReadDmg(..) => 75,
            Self::WaitRelease(..) => 85,
            Self::Upload(..) => 90,
        })
    }
}

#[cfg(windows)]
#[async_trait]
impl Task<Result<(), Error>> for BuildMacOs {
    async fn run(self) -> Result<Result<(), Error>, Self> {
        match self {
            Self::Connect(client, repo, release_rx) => gres::transpose(async move {
                let mut ssh = Command::new("ssh").arg(MACOS_ADDR).arg("/opt/git/github.com/fenhl/oottracker/master/target/release/oottracker-release").stdout(Stdio::piped()).spawn()?;
                let stdout = ssh.stdout.take().expect("stdout was piped");
                Ok(Err(Self::Remote(format!("connecting to Mac"), Percent::default(), client, repo, release_rx, ssh, stdout)))
            }).await,
            Self::Remote(_, _, client, repo, release_rx, ssh, mut stdout) => gres::transpose(async move {
                match MacMessage::read(&mut stdout).await {
                    Ok(msg) => match msg {
                        MacMessage::Progress { label, percent } => return Ok(Err(Self::Remote(label, percent, client, repo, release_rx, ssh, stdout))),
                    },
                    Err(ReadError::EndOfStream) => {}
                    Err(ReadError::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof => {}
                    Err(e) => return Err(e.into()),
                }
                Ok(Err(Self::Disconnect(client, repo, release_rx, ssh)))
            }).await,
            Self::Disconnect(client, repo, release_rx, ssh) => gres::transpose(async move {
                ssh.check("oottracker-release").await?;
                Ok(Err(Self::Download(client, repo, release_rx)))
            }).await,
            Self::Download(client, repo, release_rx) => gres::transpose(async move {
                Command::new("scp").arg(format!("{}:/opt/git/github.com/fenhl/oottracker/master/assets/oottracker-mac.dmg", MACOS_ADDR)).arg("assets/oottracker-mac.dmg").check("scp").await?;
                Ok(Err(Self::ReadDmg(client, repo, release_rx)))
            }).await,
            Self::ReadDmg(client, repo, release_rx) => gres::transpose(async move {
                let dmg_data = fs::read("assets/oottracker-mac.dmg").await?;
                Ok(Err(Self::WaitRelease(client, repo, release_rx, dmg_data)))
            }).await,
            Self::WaitRelease(client, repo, mut release_rx, dmg_data) => gres::transpose(async move {
                let release = release_rx.recv().await?;
                Ok(Err(Self::Upload(client, repo, release, dmg_data)))
            }).await,
            Self::Upload(client, repo, release, dmg_data) => gres::transpose(async move {
                repo.release_attach(&client, &release, "oottracker-mac.dmg", "application/x-apple-diskimage", dmg_data).await?;
                Ok(Ok(()))
            }).await,
        }
    }
}

#[cfg(windows)]
enum BuildPj64 {
    CompileJs(reqwest::Client, Repo, broadcast::Receiver<Release>),
    WaitRelease(reqwest::Client, Repo, broadcast::Receiver<Release>, Vec<u8>),
    Upload(reqwest::Client, Repo, Release, Vec<u8>),
}

#[cfg(windows)]
impl BuildPj64 {
    fn new(client: reqwest::Client, repo: Repo, release_rx: broadcast::Receiver<Release>) -> Self {
        Self::CompileJs(client, repo, release_rx)
    }
}

#[cfg(windows)]
impl fmt::Display for BuildPj64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CompileJs(..) => write!(f, "compiling oottracker-pj64.js"),
            Self::WaitRelease(..) => write!(f, "waiting for GitHub release to be created"),
            Self::Upload(..) => write!(f, "uploading oottracker-pj64.js"),
        }
    }
}

#[cfg(windows)]
impl Progress for BuildPj64 {
    fn progress(&self) -> Percent {
        Percent::fraction(match self {
            Self::CompileJs(..) => 0,
            Self::WaitRelease(..) => 1,
            Self::Upload(..) => 2,
        }, 3)
    }
}

#[cfg(windows)]
#[async_trait]
impl Task<Result<(), Error>> for BuildPj64 {
    async fn run(self) -> Result<Result<(), Error>, Self> {
        match self {
            Self::CompileJs(client, repo, release_rx) => gres::transpose(async move {
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
                Ok(Err(Self::WaitRelease(client, repo, release_rx, buf)))
            }).await,
            Self::WaitRelease(client, repo, mut release_rx, buf) => gres::transpose(async move {
                let release = release_rx.recv().await?;
                Ok(Err(Self::Upload(client, repo, release, buf)))
            }).await,
            Self::Upload(client, repo, release, buf) => gres::transpose(async move {
                repo.release_attach(&client, &release, "oottracker-pj64.js", "text/javascript", buf).await?;
                Ok(Ok(()))
            }).await,
        }
    }
}

#[cfg(windows)]
enum BuildWeb {
    UpdateRepo,
    Build,
    Restart,
}

#[cfg(windows)]
impl Default for BuildWeb {
    fn default() -> Self {
        Self::UpdateRepo
    }
}

#[cfg(windows)]
impl fmt::Display for BuildWeb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UpdateRepo => write!(f, "updating repo on mercredi"),
            Self::Build => write!(f, "building oottracker.fenhl.net"),
            Self::Restart => write!(f, "restarting oottracker.fenhl.net"),
        }
    }
}

#[cfg(windows)]
impl Progress for BuildWeb {
    fn progress(&self) -> Percent {
        Percent::new(match self {
            Self::UpdateRepo => 0,
            Self::Build => 10,
            Self::Restart => 95,
        })
    }
}

#[cfg(windows)]
#[async_trait]
impl Task<Result<(), Error>> for BuildWeb {
    async fn run(self) -> Result<Result<(), Error>, Self> {
        match self {
            Self::UpdateRepo => gres::transpose(async move {
                Command::new("ssh").arg("mercredi").arg("cd /opt/git/github.com/fenhl/oottracker/master && git pull --ff-only").check("ssh").await?;
                Ok(Err(Self::Build))
            }).await,
            Self::Build => gres::transpose(async move {
                Command::new("ssh").arg("mercredi").arg(concat!("env -C /opt/git/github.com/fenhl/oottracker/master ", include_str!("../../../assets/web/env.txt"), " cargo build --release --package=oottracker-web")).check("ssh").await?;
                Ok(Err(Self::Restart))
            }).await,
            Self::Restart => gres::transpose(async move {
                Command::new("ssh").arg("mercredi").arg("sudo systemctl restart oottracker-web").check("ssh").await?;
                Ok(Ok(()))
            }).await,
        }
    }
}

#[cfg(windows)]
enum CreateRelease {
    CreateNotesFile(Repo, reqwest::Client, broadcast::Sender<Release>, Arc<Cli>, Args),
    EditNotes(Repo, reqwest::Client, broadcast::Sender<Release>, Arc<Cli>, Args, NamedTempFile),
    ReadNotes(Repo, reqwest::Client, broadcast::Sender<Release>, NamedTempFile),
    Create(Repo, reqwest::Client, broadcast::Sender<Release>, String),
}

#[cfg(windows)]
impl CreateRelease {
    fn new(repo: Repo, client: reqwest::Client, tx: broadcast::Sender<Release>, cli: Arc<Cli>, args: Args) -> Self {
        Self::CreateNotesFile(repo, client, tx, cli, args)
    }
}

#[cfg(windows)]
impl fmt::Display for CreateRelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateNotesFile(..) => write!(f, "creating release notes file"),
            Self::EditNotes(..) => write!(f, "waiting for release notes"),
            Self::ReadNotes(..) => write!(f, "reading release notes"),
            Self::Create(..) => write!(f, "creating release"),
        }
    }
}

#[cfg(windows)]
impl Progress for CreateRelease {
    fn progress(&self) -> Percent {
        Percent::fraction(match self {
            Self::CreateNotesFile(..) => 0,
            Self::EditNotes(..) => 1,
            Self::ReadNotes(..) => 2,
            Self::Create(..) => 3,
        }, 4)
    }
}

#[cfg(windows)]
#[async_trait]
impl Task<Result<Release, Error>> for CreateRelease {
    async fn run(self) -> Result<Result<Release, Error>, Self> {
        match self {
            Self::CreateNotesFile(repo, client, tx, cli, args) => gres::transpose(async move {
                let notes_file = tokio::task::spawn_blocking(|| {
                    tempfile::Builder::new()
                        .prefix("oottracker-release-notes")
                        .suffix(".md")
                        .tempfile()
                }).await??;
                Ok(Err(Self::EditNotes(repo, client, tx, cli, args, notes_file)))
            }).await,
            Self::EditNotes(repo, client, tx, cli, args, notes_file) => gres::transpose(async move {
                let mut cmd;
                let (cmd_name, cli_lock) = if let Some(ref editor) = args.release_notes_editor {
                    cmd = Command::new(editor);
                    if !args.no_wait {
                        cmd.arg("--wait");
                    }
                    ("editor", Some(cli.lock().await))
                } else {
                    if env::var("TERM_PROGRAM").as_deref() == Ok("vscode") && env::var_os("STY").is_none() && env::var_os("SSH_CLIENT").is_none() && env::var_os("SSH_TTY").is_none() {
                        cmd = Command::new("C:\\Program Files\\Microsoft VS Code\\bin\\code.cmd");
                        if !args.no_wait {
                            cmd.arg("--wait");
                        }
                        ("code", None)
                    } else {
                        cmd = Command::new("C:\\ProgramData\\chocolatey\\bin\\nano.exe");
                        ("nano", Some(cli.lock().await))
                    }
                };
                cmd.arg(notes_file.path()).spawn()?.check(cmd_name).await?; // spawn before checking to avoid capturing stdio
                drop(cli_lock);
                Ok(Err(Self::ReadNotes(repo, client, tx, notes_file)))
            }).await,
            Self::ReadNotes(repo, client, tx, mut notes_file) => gres::transpose(async move {
                let notes = tokio::task::spawn_blocking(move || {
                    let mut buf = String::default();
                    notes_file.read_to_string(&mut buf)?;
                    if buf.is_empty() { return Err(Error::EmptyReleaseNotes) }
                    Ok(buf)
                }).await??;
                Ok(Err(Self::Create(repo, client, tx, notes)))
            }).await,
            Self::Create(repo, client, tx, notes) => gres::transpose(async move {
                let release = repo.create_release(&client, format!("OoT Tracker {}", version().await), format!("v{}", version().await), notes).await?;
                tx.send(release.clone())?;
                Ok(Ok(release))
            }).await,
        }
    }
}

#[cfg(windows)]
#[derive(Clone, clap::Parser)]
#[clap(version)]
struct Args {
    /// Create the GitHub release as a draft
    #[clap(long)]
    no_publish: bool,
    /// Don't pass `--wait` to the release notes editor
    #[clap(short = 'W', long)]
    no_wait: bool,
    /// the editor for the release notes
    #[clap(short = 'e', long, parse(from_os_str))]
    release_notes_editor: Option<OsString>,
}

#[cfg(target_os = "macos")]
#[wheel::main(debug)]
async fn main() -> Result<(), Error> {
    let mut stdout = stdout();
    MacMessage::Progress { label: format!("acquiring rustup lock"), percent: Percent::new(0) }.write(&mut stdout).await?;
    let lock = DirLock::new("/tmp/syncbin-startup-rust.lock").await?;
    MacMessage::Progress { label: format!("updating rustup"), percent: Percent::new(5) }.write(&mut stdout).await?;
    let mut rustup_cmd = Command::new("rustup");
    rustup_cmd.arg("self");
    rustup_cmd.arg("update");
    rustup_cmd.stdout(Stdio::null());
    if let Some(base_dirs) = BaseDirs::new() {
        rustup_cmd.env("PATH", format!("{}:{}", base_dirs.home_dir().join(".cargo").join("bin").display(), env::var("PATH")?));
    }
    rustup_cmd.check("rustup").await?;
    MacMessage::Progress { label: format!("updating Rust"), percent: Percent::new(10) }.write(&mut stdout).await?;
    let mut rustup_cmd = Command::new("rustup");
    rustup_cmd.arg("update");
    rustup_cmd.arg("stable");
    rustup_cmd.stdout(Stdio::null());
    if let Some(base_dirs) = BaseDirs::new() {
        rustup_cmd.env("PATH", format!("{}:{}", base_dirs.home_dir().join(".cargo").join("bin").display(), env::var("PATH")?));
    }
    rustup_cmd.check("rustup").await?;
    lock.drop_async().await?;
    MacMessage::Progress { label: format!("cleaning up outdated cargo build artifacts"), percent: Percent::new(20) }.write(&mut stdout).await?;
    let mut sweep_cmd = Command::new("cargo");
    sweep_cmd.arg("sweep");
    sweep_cmd.arg("--installed");
    sweep_cmd.arg("-r");
    sweep_cmd.current_dir("/opt/git");
    sweep_cmd.stdout(Stdio::null());
    if let Some(base_dirs) = BaseDirs::new() {
        sweep_cmd.env("PATH", format!("{}:{}", base_dirs.home_dir().join(".cargo").join("bin").display(), env::var("PATH")?));
    }
    sweep_cmd.check("cargo").await?;
    MacMessage::Progress { label: format!("updating oottracker repo"), percent: Percent::new(25) }.write(&mut stdout).await?;
    let repo = Repository::open("/opt/git/github.com/fenhl/oottracker/master")?;
    let mut origin = repo.find_remote("origin")?;
    origin.fetch(&["main"], None, None)?;
    repo.reset(&repo.find_branch("origin/main", BranchType::Remote)?.into_reference().peel_to_commit()?.into_object(), ResetType::Hard, None)?;
    MacMessage::Progress { label: format!("building oottracker-mac.app for x86_64"), percent: Percent::new(30) }.write(&mut stdout).await?;
    Command::new("cargo").arg("build").arg("--release").arg("--target=x86_64-apple-darwin").arg("--package=oottracker-gui").env("MACOSX_DEPLOYMENT_TARGET", "10.9").current_dir("/opt/git/github.com/fenhl/oottracker/master").check("cargo").await?;
    MacMessage::Progress { label: format!("building oottracker-mac.app for aarch64"), percent: Percent::new(60) }.write(&mut stdout).await?;
    Command::new("cargo").arg("build").arg("--release").arg("--target=aarch64-apple-darwin").arg("--package=oottracker-gui").current_dir("/opt/git/github.com/fenhl/oottracker/master").check("cargo").await?;
    MacMessage::Progress { label: format!("creating Universal macOS binary"), percent: Percent::new(90) }.write(&mut stdout).await?;
    fs::create_dir("/opt/git/github.com/fenhl/oottracker/master/assets/macos/OoT Tracker.app/Contents/MacOS").await.exist_ok()?;
    Command::new("lipo").arg("-create").arg("target/aarch64-apple-darwin/release/oottracker-gui").arg("target/x86_64-apple-darwin/release/oottracker-gui").arg("-output").arg("assets/macos/OoT Tracker.app/Contents/MacOS/oottracker-gui").current_dir("/opt/git/github.com/fenhl/oottracker/master").check("lipo").await?;
    MacMessage::Progress { label: format!("packing oottracker-mac.dmg"), percent: Percent::new(95) }.write(&mut stdout).await?;
    Command::new("hdiutil").arg("create").arg("assets/oottracker-mac.dmg").arg("-volname").arg("OoT Tracker").arg("-srcfolder").arg("assets/macos").arg("-ov").current_dir("/opt/git/github.com/fenhl/oottracker/master").check("hdiutil").await?;
    Ok(())
}

#[cfg(windows)]
#[wheel::main(debug)]
async fn main(args: Args) -> Result<(), Error> {
    let cli = Arc::new(Cli::new()?);
    let create_release_cli = Arc::clone(&cli);
    let release_notes_cli = Arc::clone(&cli);
    let (client, repo, bizhawk_version) = cli.run(Setup::default(), "pre-release checks passed").await??; // don't show release notes editor if version check could still fail
    let (release_tx, release_rx_bizhawk) = broadcast::channel(1);
    let release_rx_gui = release_tx.subscribe();
    let release_rx_macos = release_tx.subscribe();
    let release_rx_pj64 = release_tx.subscribe();
    let create_release_args = args.clone();
    let create_release_client = client.clone();
    let create_release_repo = repo.clone();
    let create_release = tokio::spawn(async move {
        create_release_cli.run(CreateRelease::new(create_release_repo, create_release_client, release_tx, release_notes_cli, create_release_args), "release created").await?
    });

    macro_rules! with_metavariable {
        ($metavariable:tt, $($token:tt)*) => { $($token)* };
    }

    macro_rules! build_tasks {
        ($($task:expr => $done:literal,)*) => {
            let ($(with_metavariable!($task, ())),*) = tokio::try_join!($(
                async { cli.run($task, $done).await? }
            ),*)?;
        };
    }

    build_tasks![
        BuildBizHawk::new(client.clone(), repo.clone(), release_rx_bizhawk, bizhawk_version) => "BizHawk build done",
        BuildGui::new(client.clone(), repo.clone(), release_rx_gui) => "Windows GUI build done",
        BuildMacOs::new(client.clone(), repo.clone(), release_rx_macos) => "macOS build done",
        BuildPj64::new(client.clone(), repo.clone(), release_rx_pj64) => "Project64 build done",
        BuildWeb::default() => "web build done",
    ];
    let release = create_release.await??;
    if !args.no_publish {
        let line = cli.new_line("[....] publishing release").await?;
        repo.publish_release(&client, release).await?;
        line.replace("[done] release published").await?;
    }
    Ok(())
}
