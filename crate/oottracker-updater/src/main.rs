#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {
    std::{
        fmt,
        io,
        path::PathBuf,
        sync::Arc,
        time::Duration,
    },
    iced::{
        Application,
        Clipboard,
        Command,
        Element,
        HorizontalAlignment,
        Length,
        Settings,
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
    image::DynamicImage,
    itertools::Itertools as _,
    open::that as open,
    tokio::{
        io::AsyncWriteExt as _,
        time::sleep,
    },
    tokio_stream::StreamExt as _,
    wheel::{
        FromArc,
        fs::File,
    },
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
    path: PathBuf,
    state: State,
    discord_invite_btn: button::State,
    discord_channel_btn: button::State,
    new_issue_btn: button::State,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Result<Message, Error>;
    type Flags = PathBuf;

    fn new(path: PathBuf) -> (Self, Command<Result<Message, Error>>) {
        (App {
            path,
            state: State::Init,
            discord_invite_btn: button::State::default(),
            discord_channel_btn: button::State::default(),
            new_issue_btn: button::State::default(),
        }, async {
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
            State::Init => Text::new("Checking latest release…").into(),
            State::WaitExit => Text::new("Waiting to make sure the old version has exited…").into(),
            State::Download => Text::new("Starting download…").into(),
            State::Replace => Text::new("Downloading update…").into(),
            State::WaitDownload => Text::new("Finishing download…").into(),
            State::Launch => Text::new("Starting new version…").into(),
            State::Done => Text::new("Closing updater…").into(),
            State::Error(ref e) => Column::new()
                .push(Text::new("Error").size(24).width(Length::Fill).horizontal_alignment(HorizontalAlignment::Center))
                .push(Text::new(e.to_string()))
                .push(Text::new(format!("debug info: {:?}", e)))
                .push(Text::new("Support").size(24).width(Length::Fill).horizontal_alignment(HorizontalAlignment::Center))
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
struct Args {
    #[clap(parse(from_os_str))]
    path: PathBuf,
}

#[derive(Debug, FromArc, Clone)]
enum Error {
    Cloned,
    #[from_arc]
    Io(Arc<io::Error>),
    MissingAsset,
    NoReleases,
    #[from_arc]
    Reqwest(Arc<reqwest::Error>),
    #[from_arc]
    Wheel(Arc<wheel::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cloned => write!(f, "clone of unexpected message kind"),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::MissingAsset => write!(f, "release does not have a download for this platform"),
            Self::NoReleases => write!(f, "there are no released versions"),
            Self::Reqwest(e) => if let Some(url) = e.url() {
                write!(f, "HTTP error at {}: {}", url, e)
            } else {
                write!(f, "HTTP error: {}", e)
            },
            Self::Wheel(e) => e.fmt(f),
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
        ..Settings::with_flags(path)
    })
}
