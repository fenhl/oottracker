use {
    std::{
        any::TypeId,
        collections::hash_map::DefaultHasher,
        fmt,
        hash::{
            Hash,
            Hasher as _,
        },
        io::{
            self,
            prelude::*,
        },
        net::Ipv4Addr,
        pin::Pin,
        sync::Arc,
        time::Duration,
    },
    async_proto::Protocol as _,
    derive_more::From,
    futures::{
        future::Future,
        stream::{
            self,
            SplitSink,
            SplitStream,
            Stream,
            StreamExt as _,
            TryStreamExt as _,
        },
    },
    itertools::Itertools as _,
    tokio::{
        net::{
            TcpListener,
            TcpStream,
            UdpSocket,
        },
        sync::Mutex,
        time::sleep,
    },
    tokio_stream::wrappers::TcpListenerStream,
    tokio_tungstenite::{
        MaybeTlsStream,
        WebSocketStream,
        tungstenite,
    },
    wheel::FromArc,
    crate::{
        ModelState,
        proto::{
            self,
            Packet,
            TCP_PORT,
        },
        ram::{
            self,
            Ram,
        },
        websocket,
    },
};

#[derive(Debug, From, FromArc, Clone)]
pub enum Error {
    CannotChangeState,
    #[from_arc]
    Io(Arc<io::Error>),
    Protocol(proto::ReadError),
    RamDecode(ram::DecodeError),
    UnexpectedWebsocketMessage,
    Websocket {
        debug: String,
        display: String,
    },
    #[from_arc]
    Write(Arc<async_proto::WriteError>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CannotChangeState => write!(f, "this type of connection is read-only"),
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Protocol(e) => e.fmt(f),
            Error::RamDecode(e) => write!(f, "error decoding game RAM: {:?}", e),
            Error::UnexpectedWebsocketMessage => write!(f, "unexpected WebSocket message kind from server"),
            Error::Websocket { display, .. } => display.fmt(f),
            Error::Write(e) => e.fmt(f),
        }
    }
}

pub trait Connection: fmt::Debug + Send + Sync {
    fn hash(&self) -> u64;
    fn can_change_state(&self) -> bool;
    fn display_kind(&self) -> &'static str;
    fn packet_stream(&self) -> Pin<Box<dyn Stream<Item = Result<Packet, Error>> + Send>>;
    fn set_state(&self, model: &ModelState) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;
}

#[derive(Debug, Clone, Copy)]
pub struct NullConnection;

impl Connection for NullConnection {
    fn hash(&self) -> u64 {
        let mut state = DefaultHasher::default();
        TypeId::of::<Self>().hash(&mut state);
        state.finish()
    }

    fn can_change_state(&self) -> bool { false }
    fn display_kind(&self) -> &'static str { "nothing" }

    fn packet_stream(&self) -> Pin<Box<dyn Stream<Item = Result<Packet, Error>> + Send>> {
        Box::pin(stream::pending())
    }

    fn set_state(&self, _: &ModelState) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> {
        Box::pin(async { Err(Error::CannotChangeState) })
    }
}

type WsStream = Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>;
type WsSink = Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>>>;

pub struct WebConnection {
    room: String,
    sink: WsSink,
    stream: WsStream,
}

impl WebConnection {
    pub async fn new(room: impl ToString) -> Result<WebConnection, async_proto::WriteError> {
        let (mut sink, stream) = tokio_tungstenite::connect_async("wss://oottracker.fenhl.net/websocket").await.map_err(|e| async_proto::WriteError {
            context: async_proto::ErrorContext::Custom(format!("oottracker::net::WebConnection::new")),
            kind: e.into(),
        })?.0.split();
        websocket::ClientMessage::SubscribeRaw { room: room.to_string() }.write_ws021(&mut sink).await?;
        Ok(WebConnection {
            room: room.to_string(),
            sink: Arc::new(Mutex::new(sink)),
            stream: Arc::new(Mutex::new(stream)),
        })
    }
}

impl fmt::Debug for WebConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WebConnection {{ room: {:?}, .. }}", self.room) //TODO finish_non_exhaustive
    }
}

impl Connection for WebConnection {
    fn hash(&self) -> u64 {
        let mut state = DefaultHasher::default();
        TypeId::of::<Self>().hash(&mut state);
        self.room.hash(&mut state);
        state.finish()
    }

    fn can_change_state(&self) -> bool { true } //TODO support for read-only (passwordless) connections?
    fn display_kind(&self) -> &'static str { "web" }

    fn packet_stream(&self) -> Pin<Box<dyn Stream<Item = Result<Packet, Error>> + Send>> {
        let stream = Arc::clone(&self.stream);
        Box::pin(stream::unfold(stream, |stream| async move {
            loop {
                let stream_clone = Arc::clone(&stream);
                break match websocket::ServerMessage::read_ws021(&mut *stream_clone.lock().await).await {
                    Ok(websocket::ServerMessage::Ping) => continue,
                    Ok(websocket::ServerMessage::Error { debug, display }) => Some((Err(Error::Websocket { debug, display }), stream)),
                    Ok(websocket::ServerMessage::Init(_)) | Ok(websocket::ServerMessage::Update { .. }) => Some((Err(Error::UnexpectedWebsocketMessage), stream)),
                    Ok(websocket::ServerMessage::InitRaw(model)) => Some((Ok(Packet::ModelInit(model)), stream)),
                    Ok(websocket::ServerMessage::UpdateRaw(delta)) => Some((Ok(Packet::ModelDelta(delta)), stream)),
                    Err(e) => Some((Err(Error::Protocol(proto::ReadError::Packet(Arc::new(e)))), stream)),
                };
            }
        }))
    }

    fn set_state(&self, model: &ModelState) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> {
        let room = self.room.clone();
        let state = model.clone();
        let sink = Arc::clone(&self.sink);
        Box::pin(async move {
            websocket::ClientMessage::SetRaw { room, state }
                .write_ws021(&mut *sink.lock().await)
                .await?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TcpConnection;

impl Connection for TcpConnection {
    fn hash(&self) -> u64 {
        let mut state = DefaultHasher::default();
        TypeId::of::<Self>().hash(&mut state);
        state.finish()
    }

    fn can_change_state(&self) -> bool { false } //TODO support for two-way TCP connections?
    fn display_kind(&self) -> &'static str { "TCP" }

    fn packet_stream(&self) -> Pin<Box<dyn Stream<Item = Result<Packet, Error>> + Send>> {
        Box::pin(
            stream::once(async { TcpListener::bind((Ipv4Addr::LOCALHOST, TCP_PORT)).await })
                .map_ok(|listener| TcpListenerStream::new(listener).err_into::<Error>())
                .try_flatten()
                .map_ok(|tcp_stream| proto::read(tcp_stream).err_into::<Error>())
                .try_flatten()
        )
    }

    fn set_state(&self, _: &ModelState) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> {
        Box::pin(async { Err(Error::CannotChangeState) })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RetroArchConnection {
    pub port: u16,
}

impl Connection for RetroArchConnection {
    fn hash(&self) -> u64 {
        let mut state = DefaultHasher::default();
        TypeId::of::<Self>().hash(&mut state);
        self.port.hash(&mut state);
        state.finish()
    }

    fn can_change_state(&self) -> bool { false }
    fn display_kind(&self) -> &'static str { "RetroArch" }

    fn packet_stream(&self) -> Pin<Box<dyn Stream<Item = Result<Packet, Error>> + Send>> {
        let port = self.port;
        Box::pin(stream::try_unfold(Box::pin(async move {
            let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
            sock.connect((Ipv4Addr::LOCALHOST, port)).await?;
            Ok::<_, Error>(sock)
        }) as Pin<Box<dyn Future<Output = _> + Send>>, |sock| async move {
            sleep(Duration::from_secs(1)).await;
            let sock = sock.await?;
            let ram = retroarch_read_ram(&sock).await?;
            Ok(Some((Packet::RamInit(ram), Box::pin(async move { Ok(sock) }) as Pin<Box<dyn Future<Output = _> + Send>>)))
        }))
    }

    fn set_state(&self, _: &ModelState) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> {
        Box::pin(async { Err(Error::CannotChangeState) })
    }
}

/// The RetroArch UDP API does not seem to be documented,
/// but there is a Python implementation at
/// <https://github.com/eadmaster/console_hiscore/blob/master/tools/retroarchpythonapi.py>
async fn retroarch_read_ram(sock: &UdpSocket) -> Result<Ram, Error> {
    let ranges = stream::iter(ram::RANGES.iter().copied().tuples()).then(|(start, len)| async move {
        let start = 0x8000_0000 + start; // ram::RANGES uses RDRAM addresses but READ_CORE_MEMORY uses system bus addresses
        // make sure we're word-aligned on both ends
        let offset_in_word = start & 0x3;
        let mut aligned_start = (start - offset_in_word) as usize;
        let mut aligned_len = len + offset_in_word;
        if aligned_len % 0x3 != 0 { aligned_len += 4 - (aligned_len & 0x3) }
        let mut packet_buf = [0; 4096];
        let mut ram_buf = Vec::with_capacity(aligned_len as usize);
        let mut prefix = Vec::with_capacity(21);
        let mut msg = Vec::with_capacity(26);
        while aligned_len > 0 {
            // make sure the hex-encoded response fits into the 4096-byte buffer RetroArch uses
            // each encoded byte requires 3 bytes of buffer space (the whitespace plus the 2-character hex encoding)
            const MAX_ENCODED_BYTES_PER_BUFFER: u32 = (4_096 - "READ_CORE_MEMORY ffffffff 9999\n".len() as u32) / 3;

            // using READ_CORE_MEMORY instead of READ_CORE_RAM as suggested in https://github.com/libretro/RetroArch/blob/0357b6c/command.h#L430-L437
            let count = aligned_len.min(MAX_ENCODED_BYTES_PER_BUFFER);
            prefix.clear();
            write!(&mut prefix, "READ_CORE_MEMORY {:x} ", aligned_start).expect("failed to compose packet");
            msg.clear();
            write!(&mut msg, "READ_CORE_MEMORY {:x} ", aligned_start).expect("failed to compose packet");
            writeln!(&mut msg, "{}", count).expect("failed to compose packet");
            sock.send(&msg).await?;
            let packet_len = sock.recv(&mut packet_buf).await?;
            let response = &packet_buf[prefix.len()..packet_len - 1];
            let words = response.split(|&sep| sep == b' ').map(|byte| u8::from_str_radix(&String::from_utf8_lossy(byte), 16).expect("invalid byte representation")).tuples();
            for (b3, b2, b1, b0) in words {
                ram_buf.extend_from_slice(&[b0, b1, b2, b3]);
            }
            //if words.into_buffer().next().is_some() { panic!("did not receive a whole number of words") }
            aligned_start += count as usize;
            aligned_len -= count;
        }
        Ok::<Vec<u8>, Error>(ram_buf[offset_in_word as usize..(offset_in_word + len) as usize].to_owned())
    }).try_collect::<Vec<_>>().await?;
    Ok(Ram::from_range_bufs(ranges)?)
}
