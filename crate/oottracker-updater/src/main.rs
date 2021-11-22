#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {
    std::{
        fmt,
        io,
        path::PathBuf,
        time::Duration,
    },
    derive_more::From,
    iced::{
        Application,
        Clipboard,
        Command,
        Element,
        Settings,
        widget::Text,
        window::{
            self,
            Icon,
        },
    },
    image::DynamicImage,
    itertools::Itertools as _,
    structopt::StructOpt,
    tokio::{
        fs::File,
        io::AsyncWriteExt as _,
        time::sleep,
    },
    tokio_stream::StreamExt as _,
    oottracker::{
        github::{
            ReleaseAsset,
            Repo,
        },
        ui::images,
    },
};

#[cfg(target_arch = "x86")]
const PLATFORM_SUFFIX: &str = "-win32.exe";
#[cfg(target_arch = "x86_64")]
const PLATFORM_SUFFIX: &str = "-win64.exe";

enum State {
    Init,
    WaitExit,
    Download,
    Replace,
    WaitDownload,
    Launch,
    Done,
    Error(Error),
}

#[derive(Debug)]
enum Message {
    ReleaseAsset(reqwest::Client, ReleaseAsset),
    WaitedExit(reqwest::Client, ReleaseAsset),
    Response(reqwest::Response),
    Downloaded(File),
    WaitedDownload,
    Done,
}

struct App {
    path: PathBuf,
    state: State,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Result<Message, Error>;
    type Flags = PathBuf;

    fn new(path: PathBuf) -> (Self, Command<Result<Message, Error>>) {
        (App { path, state: State::Init }, async {
            let client = reqwest::Client::builder()
                .user_agent(concat!("oottracker-updater/", env!("CARGO_PKG_VERSION")))
                .build()?;
            let release = Repo::new("fenhl", "oottracker").latest_release(&client).await?.ok_or(Error::NoReleases)?;
            let (asset,) = release.assets.into_iter()
                .filter(|asset| asset.name.ends_with(PLATFORM_SUFFIX))
                .collect_tuple().ok_or(Error::MissingAsset)?;
            Ok(Message::ReleaseAsset(client, asset))
        }.into())
    }

    fn title(&self) -> String { format!("updating the OoT tracker…") }

    fn update(&mut self, msg: Result<Message, Error>, _: &mut Clipboard) -> Command<Result<Message, Error>> {
        match msg {
            Ok(Message::ReleaseAsset(client, asset)) => {
                self.state = State::WaitExit;
                async {
                    sleep(Duration::from_secs(1)).await;
                    Ok(Message::WaitedExit(client, asset))
                }.into()
            }
            Ok(Message::WaitedExit(client, asset)) => {
                self.state = State::Download;
                async move {
                    Ok(Message::Response(client.get(asset.browser_download_url).send().await?.error_for_status()?))
                }.into()
            }
            Ok(Message::Response(response)) => {
                self.state = State::Replace;
                let path = self.path.clone();
                async move {
                    let mut data = response.bytes_stream();
                    let mut exe_file = File::create(path).await?;
                    while let Some(chunk) = data.try_next().await? {
                        exe_file.write_all(chunk.as_ref()).await?;
                    }
                    Ok(Message::Downloaded(exe_file))
                }.into()
            }
            Ok(Message::Downloaded(exe_file)) => {
                self.state = State::WaitDownload;
                async move {
                    exe_file.sync_all().await?;
                    Ok(Message::WaitedDownload)
                }.into()
            }
            Ok(Message::WaitedDownload) => {
                self.state = State::Launch;
                let path = self.path.clone();
                async move {
                    std::process::Command::new(path).spawn()?;
                    Ok(Message::Done)
                }.into()
            }
            Ok(Message::Done) => {
                self.state = State::Done;
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
            State::Init => Text::new("Checking latest release…").into(),
            State::WaitExit => Text::new("Waiting to make sure the old version has exited…").into(),
            State::Download => Text::new("Starting download…").into(),
            State::Replace => Text::new("Downloading update…").into(),
            State::WaitDownload => Text::new("Finishing download…").into(),
            State::Launch => Text::new("Starting new version…").into(),
            State::Done => Text::new("Closing updater…").into(),
            State::Error(ref e) => Text::new(format!("error: {}", e)).into(),
        }
    }

    fn should_exit(&self) -> bool {
        matches!(self.state, State::Done)
    }
}

#[derive(StructOpt)]
struct Args {
    path: PathBuf,
}

#[derive(Debug, From)]
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
fn main(Args { path }: Args) -> iced::Result {
    let icon = images::icon::<DynamicImage>().to_rgba8();
    App::run(Settings {
        window: window::Settings {
            size: (320, 240),
            icon: Icon::from_rgba(icon.as_flat_samples().as_slice().to_owned(), icon.width(), icon.height()).ok(), // simply omit the icon if loading it fails
            ..window::Settings::default()
        },
        flags: path,
        ..Settings::default()
    })
}
