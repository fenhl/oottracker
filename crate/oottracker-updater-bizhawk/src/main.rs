#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_qualifications, warnings)]

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {
    std::{
        cmp::Ordering::*,
        ffi::OsString,
        fmt,
        iter,
        os::windows::ffi::{
            OsStrExt as _,
            OsStringExt as _,
        },
        path::PathBuf,
        ptr::null_mut,
        sync::Arc,
        time::Duration,
    },
    async_zip::error::ZipError,
    bytes::Bytes,
    derive_more::From,
    futures::{
        future::FutureExt as _,
        stream::TryStreamExt as _,
    },
    iced::{
        Application,
        Command,
        Element,
        Length,
        Settings,
        alignment,
        widget::{
            button::{
                self,
                Button,
            },
            Column,
            Row,
            Text,
        },
        window::{
            self,
            Icon,
        },
    },
    iced_native::command::Action,
    image::DynamicImage,
    itertools::Itertools as _,
    open::that as open,
    semver::Version,
    sysinfo::{
        Pid,
        ProcessRefreshKind,
        SystemExt as _,
    },
    tokio::{
        io,
        time::sleep,
    },
    tokio_util::io::StreamReader,
    wheel::{
        FromArc,
        fs::{
            self,
            File,
        },
    },
    winapi::um::fileapi::GetFullPathNameW,
    oottracker::{
        github::{
            ReleaseAsset,
            Repo,
        },
        ui::images,
    },
};

#[cfg(target_arch = "x86")] const TRACKER_PLATFORM_SUFFIX: &str = "-bizhawk-win32.zip";
#[cfg(target_arch = "x86_64")] const TRACKER_PLATFORM_SUFFIX: &str = "-bizhawk-win64.zip";

#[cfg(target_arch = "x86_64")] const BIZHAWK_PLATFORM_SUFFIX: &str = "-win-x64.zip";

enum State {
    WaitExit,
    GetTrackerRelease,
    DownloadTracker,
    ExtractTracker,
    GetBizHawkRelease,
    StartDownloadBizHawk,
    DownloadBizHawk,
    ExtractBizHawk,
    Launch,
    Done,
    Error(Error),
}

#[derive(Debug)]
enum Message {
    Exited,
    TrackerReleaseAsset(reqwest::Client, ReleaseAsset),
    TrackerResponse(reqwest::Client, reqwest::Response),
    UpdateBizHawk(reqwest::Client, Version),
    BizHawkReleaseAsset(reqwest::Client, ReleaseAsset),
    BizHawkResponse(reqwest::Response),
    BizHawkZip(Bytes),
    Launch,
    Done,
    DiscordInvite,
    DiscordChannel,
    NewIssue,
    Cloned,
}

impl Clone for Message {
    fn clone(&self) -> Self {
        match self {
            Self::DiscordInvite => Self::DiscordInvite,
            Self::DiscordChannel => Self::DiscordChannel,
            Self::NewIssue => Self::NewIssue,
            _ => Self::Cloned,
        }
    }
}

struct App {
    args: Args,
    state: State,
    discord_invite_btn: button::State,
    discord_channel_btn: button::State,
    new_issue_btn: button::State,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Result<Message, Error>;
    type Flags = Args;

    fn new(args: Args) -> (Self, Command<Result<Message, Error>>) {
        let cmd = Command::single(Action::Future(async move {
            let mut system = sysinfo::System::default();
            while system.refresh_process_specifics(args.pid, ProcessRefreshKind::default()) {
                sleep(Duration::from_secs(1)).await;
            }
            Ok(Message::Exited)
        }.boxed()));
        (App {
            args,
            state: State::WaitExit,
            discord_invite_btn: button::State::default(),
            discord_channel_btn: button::State::default(),
            new_issue_btn: button::State::default(),
        }, cmd)
    }

    fn title(&self) -> String { format!("updating the OoT auto-tracker…") }

    fn update(&mut self, msg: Result<Message, Error>) -> Command<Result<Message, Error>> {
        match msg {
            Ok(Message::Exited) => {
                self.state = State::GetTrackerRelease;
                Command::single(Action::Future(async {
                    let client = reqwest::Client::builder()
                        .user_agent(concat!("oottracker-updater-bizhawk/", env!("CARGO_PKG_VERSION")))
                        .build()?;
                    let release = Repo::new("fenhl", "oottracker").latest_release(&client).await?.ok_or(Error::NoReleases)?;
                    let (asset,) = release.assets.into_iter()
                        .filter(|asset| asset.name.ends_with(TRACKER_PLATFORM_SUFFIX))
                        .collect_tuple().ok_or(Error::MissingAsset)?;
                    Ok(Message::TrackerReleaseAsset(client, asset))
                }.boxed()))
            }
            Ok(Message::TrackerReleaseAsset(client, asset)) => {
                self.state = State::DownloadTracker;
                Command::single(Action::Future(async move {
                    Ok(Message::TrackerResponse(client.clone(), client.get(asset.browser_download_url).send().await?.error_for_status()?))
                }.boxed()))
            }
            Ok(Message::TrackerResponse(client, response)) => {
                self.state = State::ExtractTracker;
                let path = self.args.path.clone();
                let local_bizhawk_version = self.args.local_bizhawk_version.clone();
                Command::single(Action::Future(async move {
                    let mut zip_file = StreamReader::new(response.bytes_stream().map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
                    let mut zip_file = async_zip::read::stream::ZipFileReader::new(&mut zip_file);
                    let mut required_bizhawk_version = None;
                    while let Some(entry) = zip_file.entry_reader().await? {
                        match entry.entry().name() {
                            "README.txt" => {
                                let (readme_prefix, _) = include_str!("../../../assets/bizhawk-readme.txt").split_once("{}").expect("failed to parse readme template");
                                required_bizhawk_version = Some(
                                    entry.read_to_string_crc().await?
                                        .strip_prefix(readme_prefix).ok_or(Error::ReadmeFormat)?
                                        .split_once(". ").ok_or(Error::ReadmeFormat)?
                                        .0.parse()?
                                );
                            }
                            "OotAutoTracker.dll" => {
                                let external_tools = path.join("ExternalTools");
                                fs::create_dir_all(&external_tools).await?;
                                entry.copy_to_end_crc(&mut File::create(external_tools.join("OotAutoTracker.dll")).await?, 64 * 1024).await?;
                            }
                            "oottracker.dll" => {
                                let external_tools = path.join("ExternalTools");
                                fs::create_dir_all(&external_tools).await?;
                                entry.copy_to_end_crc(&mut File::create(external_tools.join("oottracker.dll")).await?, 64 * 1024).await?;
                            }
                            _ => return Err(Error::UnexpectedZipEntry),
                        }
                    }
                    let required_bizhawk_version = required_bizhawk_version.ok_or(Error::MissingReadme)?;
                    match local_bizhawk_version.cmp(&required_bizhawk_version) {
                        Less => Ok(Message::UpdateBizHawk(client, required_bizhawk_version)),
                        Equal => Ok(Message::Launch),
                        Greater => Err(Error::BizHawkVersionRegression),
                    }
                }.boxed()))
            }
            Ok(Message::UpdateBizHawk(client, required_version)) => {
                self.state = State::GetBizHawkRelease;
                Command::single(Action::Future(async move {
                    //TODO also update prereqs
                    let version_str = required_version.to_string();
                    let version_str = version_str.trim_end_matches(".0");
                    let release = Repo::new("TASEmulators", "BizHawk").release_by_tag(&client, version_str).await?.ok_or(Error::NoReleases)?;
                    let (asset,) = release.assets.into_iter()
                        .filter(|asset| asset.name.ends_with(BIZHAWK_PLATFORM_SUFFIX))
                        .collect_tuple().ok_or(Error::MissingAsset)?;
                    Ok(Message::BizHawkReleaseAsset(client, asset))
                }.boxed()))
            }
            Ok(Message::BizHawkReleaseAsset(client, asset)) => {
                self.state = State::StartDownloadBizHawk;
                Command::single(Action::Future(async move {
                    Ok(Message::BizHawkResponse(client.get(asset.browser_download_url).send().await?.error_for_status()?))
                }.boxed()))
            }
            Ok(Message::BizHawkResponse(response)) => {
                self.state = State::DownloadBizHawk;
                Command::single(Action::Future(async move {
                    Ok(Message::BizHawkZip(response.bytes().await?))
                }.boxed()))
            }
            Ok(Message::BizHawkZip(mut response)) => {
                self.state = State::ExtractBizHawk;
                let path = self.args.path.clone();
                Command::single(Action::Future(async move {
                    let mut zip_file = async_zip::read::mem::ZipFileReader::new(&mut response).await?;
                    let entries = zip_file.entries().iter().enumerate().map(|(idx, entry)| (idx, entry.dir(), path.join(entry.name()))).collect_vec();
                    for (idx, is_dir, path) in entries {
                        if is_dir {
                            fs::create_dir_all(path).await?;
                        } else {
                            if let Some(parent) = path.parent() {
                                fs::create_dir_all(parent).await?;
                            }
                            zip_file.entry_reader(idx).await?.copy_to_end_crc(&mut File::create(path).await?, 64 * 1024).await?;
                        }
                    }
                    Ok(Message::Launch)
                }.boxed()))
            }
            Ok(Message::Launch) => {
                self.state = State::Launch;
                let path = self.args.path.clone();
                let path_wide = path.as_os_str().encode_wide().chain(iter::once(0)).collect_vec();
                Command::single(Action::Future(async move {
                    let path = unsafe {
                        let mut buf = vec![0; 260];
                        let result = GetFullPathNameW(path_wide.as_ptr(), buf.len().try_into().expect("buffer too large"), buf.as_mut_ptr(), null_mut());
                        PathBuf::from(if result == 0 {
                            drop(path_wide);
                            return Err(Error::Io(Arc::new(io::Error::last_os_error())))
                        } else if result > u32::try_from(buf.len()).expect("buffer too large") {
                            buf = vec![0; result.try_into().expect("path too long")];
                            let result = GetFullPathNameW(path_wide.as_ptr(), buf.len().try_into().expect("buffer too large"), buf.as_mut_ptr(), null_mut());
                            drop(path_wide);
                            if result == 0 {
                                return Err(Error::Io(Arc::new(io::Error::last_os_error())))
                            } else if result > u32::try_from(buf.len()).expect("buffer too large") {
                                panic!("path too long")
                            } else {
                                OsString::from_wide(&buf[0..result.try_into().expect("path too long")])
                            }
                        } else {
                            drop(path_wide);
                            OsString::from_wide(&buf[0..result.try_into().expect("path too long")])
                        })
                    };
                    std::process::Command::new(path.join("EmuHawk.exe")).arg("--open-ext-tool-dll=OotAutoTracker").current_dir(path).spawn()?;
                    Ok(Message::Done)
                }.boxed()))
            }
            Ok(Message::Done) => {
                self.state = State::Done;
                Command::none()
            }
            Ok(Message::DiscordInvite) => {
                if let Err(e) = open("https://discord.gg/BGRrKKn") {
                    self.state = State::Error(e.into());
                }
                Command::none()
            }
            Ok(Message::DiscordChannel) => {
                if let Err(e) = open("https://discord.com/channels/274180765816848384/476723801032491008") {
                    self.state = State::Error(e.into());
                }
                Command::none()
            }
            Ok(Message::NewIssue) => {
                if let Err(e) = open("https://github.com/fenhl/oottracker/issues/new") {
                    self.state = State::Error(e.into());
                }
                Command::none()
            }
            Ok(Message::Cloned) => {
                self.state = State::Error(Error::Cloned);
                Command::none()
            }
            Err(e) => {
                self.state = State::Error(e);
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<'_, Result<Message, Error>> {
        match self.state {
            State::WaitExit => Column::new()
                .push(Text::new("An update for the OoT auto-tracker for BizHawk is available."))
                .push(Text::new("Please close BizHawk to start the update."))
                .into(),
            State::GetTrackerRelease => Text::new("Checking latest tracker release…").into(),
            State::DownloadTracker => Text::new("Starting tracker download…").into(),
            State::ExtractTracker => Text::new("Downloading and extracting tracker…").into(),
            State::GetBizHawkRelease => Text::new("Getting BizHawk download link…").into(),
            State::StartDownloadBizHawk => Text::new("Starting BizHawk download…").into(),
            State::DownloadBizHawk => Text::new("Downloading BizHawk…").into(),
            State::ExtractBizHawk => Text::new("Extracting BizHawk…").into(),
            State::Launch => Text::new("Starting new version…").into(),
            State::Done => Text::new("Closing updater…").into(),
            State::Error(ref e) => Column::new()
                .push(Text::new("Error").size(24).width(Length::Fill).horizontal_alignment(alignment::Horizontal::Center))
                .push(Text::new(e.to_string()))
                .push(Text::new(format!("debug info: {:?}", e)))
                .push(Text::new("Support").size(24).width(Length::Fill).horizontal_alignment(alignment::Horizontal::Center))
                .push(Text::new("• Ask in #setup-support on the OoT Randomizer Discord. Feel free to ping @Fenhl#4813."))
                .push(Row::new()
                    .push(Button::new(&mut self.discord_invite_btn, Text::new("invite link")).on_press(Ok(Message::DiscordInvite)))
                    .push(Button::new(&mut self.discord_channel_btn, Text::new("direct channel link")).on_press(Ok(Message::DiscordChannel)))
                )
                .push(Row::new()
                    .push(Text::new("• Or "))
                    .push(Button::new(&mut self.new_issue_btn, Text::new("open an issue")).on_press(Ok(Message::NewIssue)))
                )
                .into(),
        }
    }

    fn should_exit(&self) -> bool {
        matches!(self.state, State::Done)
    }
}

#[derive(clap::Parser)]
#[clap(version)]
struct Args {
    path: PathBuf,
    pid: Pid,
    local_bizhawk_version: Version,
}

#[derive(Debug, From, FromArc, Clone)]
enum Error {
    BizHawkVersionRegression,
    Cloned,
    #[from_arc]
    Io(Arc<io::Error>),
    MissingAsset,
    MissingReadme,
    NoReleases,
    ReadmeFormat,
    #[from_arc]
    Reqwest(Arc<reqwest::Error>),
    #[from_arc]
    SemVer(Arc<semver::Error>),
    UnexpectedZipEntry,
    #[from_arc]
    Wheel(Arc<wheel::Error>),
    #[from_arc]
    Zip(Arc<ZipError>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BizHawkVersionRegression => write!(f, "The update requires an older version of BizHawk. Update manually at your own risk, or ask Fenhl to release a new version."),
            Self::Cloned => write!(f, "clone of unexpected message kind"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::MissingAsset => write!(f, "release does not have a download for this platform"),
            Self::MissingReadme => write!(f, "the file README.md is missing from the download"),
            Self::NoReleases => write!(f, "there are no released versions"),
            Self::ReadmeFormat => write!(f, "could not find expected BizHawk version in README.md"),
            Self::Reqwest(e) => if let Some(url) = e.url() {
                write!(f, "HTTP error at {url}: {e}")
            } else {
                write!(f, "HTTP error: {e}")
            },
            Self::SemVer(e) => write!(f, "failed to parse expected BizHawk version: {e}"),
            Self::UnexpectedZipEntry => write!(f, "unexpected file in zip archive"),
            Self::Wheel(e) => e.fmt(f),
            Self::Zip(e) => write!(f, "error reading zip file: {e}"),
        }
    }
}

#[wheel::main]
fn main(args: Args) -> iced::Result {
    let icon = images::icon::<DynamicImage>().to_rgba8();
    App::run(Settings {
        window: window::Settings {
            size: (320, 240),
            icon: Icon::from_rgba(icon.as_flat_samples().as_slice().to_owned(), icon.width(), icon.height()).ok(), // simply omit the icon if loading it fails
            ..window::Settings::default()
        },
        ..Settings::with_flags(args)
    })
}
