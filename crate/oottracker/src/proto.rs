use {
    std::{
        cmp::Ordering::*,
        collections::HashMap,
        fmt,
        hash::Hash,
        io::{
            self,
            prelude::*,
        },
        sync::Arc,
    },
    async_stream::try_stream,
    async_trait::async_trait,
    byteorder::{
        NetworkEndian,
        WriteBytesExt as _,
    },
    derive_more::From,
    futures::prelude::*,
    pin_utils::pin_mut,
    tokio::{
        net::TcpStream,
        prelude::*,
    },
    crate::{
        knowledge,
        save,
    },
};

pub const TCP_PORT: u16 = 24801;
pub const VERSION: u8 = 1;

#[async_trait]
pub trait Protocol: Sized {
    type ReadError;

    async fn read(tcp_stream: &mut TcpStream) -> Result<Self, Self::ReadError>;
    async fn write(&self, tcp_stream: &mut TcpStream) -> io::Result<()>;
    fn write_sync(&self, tcp_stream: &mut std::net::TcpStream) -> io::Result<()>;
}

macro_rules! impl_protocol_primitive {
    ($ty:ty, $read:ident, $write:ident$(, $endian:ty)?) => {
        #[async_trait]
        impl Protocol for $ty {
            type ReadError = io::Error;

            async fn read(tcp_stream: &mut TcpStream) -> io::Result<$ty> {
                tcp_stream.$read().await
            }

            async fn write(&self, tcp_stream: &mut TcpStream) -> io::Result<()> {
                tcp_stream.$write(*self).await
            }

            fn write_sync(&self, tcp_stream: &mut std::net::TcpStream) -> io::Result<()> {
                tcp_stream.$write$(::<$endian>)?(*self)
            }
        }
    };
}

impl_protocol_primitive!(u8, read_u8, write_u8);
impl_protocol_primitive!(u16, read_u16, write_u16, NetworkEndian);
impl_protocol_primitive!(u64, read_u64, write_u64, NetworkEndian);

#[derive(Debug)]
struct BoolReadError(u8);

impl fmt::Display for BoolReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected bool (0 or 1) but received {}", self.0)
    }
}

impl std::error::Error for BoolReadError {}

#[async_trait]
impl Protocol for bool {
    type ReadError = io::Error;

    async fn read(tcp_stream: &mut TcpStream) -> io::Result<bool> {
        Ok(match u8::read(tcp_stream).await? {
            0 => false,
            1 => true,
            n => return Err(io::Error::new(io::ErrorKind::InvalidData, Box::new(BoolReadError(n)))),
        })
    }

    async fn write(&self, tcp_stream: &mut TcpStream) -> io::Result<()> {
        if *self { 1u8 } else { 0 }.write(tcp_stream).await
    }

    fn write_sync(&self, tcp_stream: &mut std::net::TcpStream) -> io::Result<()> {
        if *self { 1u8 } else { 0 }.write_sync(tcp_stream)
    }
}

#[derive(Debug)]
struct StringReadEof;

impl fmt::Display for StringReadEof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "reached end of stream while reading string")
    }
}

impl std::error::Error for StringReadEof {}

#[async_trait]
impl Protocol for String {
    type ReadError = io::Error;

    async fn read(tcp_stream: &mut TcpStream) -> io::Result<String> {
        let len = u64::read(tcp_stream).await?;
        let mut buf = String::default();
        tcp_stream.take(len).read_to_string(&mut buf).await?;
        if buf.len() as u64 != len { return Err(io::Error::new(io::ErrorKind::UnexpectedEof, StringReadEof)) }
        Ok(buf)
    }

    async fn write(&self, tcp_stream: &mut TcpStream) -> io::Result<()> {
        (self.len() as u64).write(tcp_stream).await?;
        tcp_stream.write_all(self.as_bytes()).await?;
        Ok(())
    }

    fn write_sync(&self, tcp_stream: &mut std::net::TcpStream) -> io::Result<()> {
        (self.len() as u64).write_sync(tcp_stream)?;
        tcp_stream.write_all(self.as_bytes())?;
        Ok(())
    }
}

#[async_trait]
impl<T: Protocol + Sync + 'static> Protocol for Option<T>
where T::ReadError: From<io::Error> {
    type ReadError = T::ReadError;

    async fn read(tcp_stream: &mut TcpStream) -> Result<Option<T>, T::ReadError> {
        Ok(if bool::read(tcp_stream).await? {
            Some(T::read(tcp_stream).await?)
        } else {
            None
        })
    }

    async fn write(&self, tcp_stream: &mut TcpStream) -> io::Result<()> {
        if let Some(value) = self {
            true.write(tcp_stream).await?;
            value.write(tcp_stream).await?;
        } else {
            false.write(tcp_stream).await?;
        }
        Ok(())
    }

    fn write_sync(&self, tcp_stream: &mut std::net::TcpStream) -> io::Result<()> {
        if let Some(value) = self {
            true.write_sync(tcp_stream)?;
            value.write_sync(tcp_stream)?;
        } else {
            false.write_sync(tcp_stream)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum HashMapReadError<K: Protocol, V: Protocol> {
    Length(<u64 as Protocol>::ReadError),
    Key(K::ReadError),
    Value(V::ReadError),
}

impl<K: Protocol, V: Protocol> fmt::Display for HashMapReadError<K, V>
where K::ReadError: fmt::Display, V::ReadError: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashMapReadError::Length(e) => write!(f, "failed to read HashMap length: {}", e),
            HashMapReadError::Key(e) => write!(f, "failed to read key: {}", e),
            HashMapReadError::Value(e) => write!(f, "failed to read value: {}", e),
        }
    }
}

#[cfg(any(target_pointer_width = "16", target_pointer_width = "32", target_pointer_width = "64"))] // if pointer width > 64, usize doesn't fit into u64, so write can fail
#[async_trait]
impl<K: Protocol + Eq + Hash + Send + Sync + 'static, V: Protocol + Send + Sync + 'static> Protocol for HashMap<K, V>
where K::ReadError: Send, V::ReadError: Send {
    type ReadError = HashMapReadError<K, V>;

    async fn read(tcp_stream: &mut TcpStream) -> Result<HashMap<K, V>, HashMapReadError<K, V>> {
        let mut map = HashMap::default();
        for _ in 0..u64::read(tcp_stream).await.map_err(HashMapReadError::Length)? {
            map.insert(K::read(tcp_stream).await.map_err(HashMapReadError::Key)?, V::read(tcp_stream).await.map_err(HashMapReadError::Value)?);
        }
        Ok(map)
    }

    async fn write(&self, tcp_stream: &mut TcpStream) -> io::Result<()> {
        (self.len() as u64).write(tcp_stream).await?;
        for (k, v) in self {
            k.write(tcp_stream).await?;
            v.write(tcp_stream).await?;
        }
        Ok(())
    }

    fn write_sync(&self, tcp_stream: &mut std::net::TcpStream) -> io::Result<()> {
        (self.len() as u64).write_sync(tcp_stream)?;
        for (k, v) in self {
            k.write_sync(tcp_stream)?;
            v.write_sync(tcp_stream)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Packet {
    Goodbye,
    SaveDelta(save::Delta),
    SaveInit(save::Save),
    KnowledgeInit(knowledge::Knowledge),
}

#[derive(Debug, From, Clone)]
pub enum PacketReadError {
    Io(Arc<io::Error>),
    #[from]
    Knowledge(knowledge::KnowledgeReadError),
    #[from]
    SaveData(save::SaveDataReadError),
    UnknownPacketId(u8),
}

impl From<io::Error> for PacketReadError {
    fn from(e: io::Error) -> PacketReadError {
        PacketReadError::Io(Arc::new(e))
    }
}

impl fmt::Display for PacketReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PacketReadError::Io(e) => write!(f, "I/O error: {}", e),
            PacketReadError::Knowledge(e) => write!(f, "failed to decode knowledge data: {}", e),
            PacketReadError::SaveData(e) => write!(f, "failed to decode save data: {}", e),
            PacketReadError::UnknownPacketId(id) => write!(f, "unknown packet ID: {}", id),
        }
    }
}

#[async_trait]
impl Protocol for Packet {
    type ReadError = PacketReadError;

    async fn read(tcp_stream: &mut TcpStream) -> Result<Packet, PacketReadError> {
        Ok(match u8::read(tcp_stream).await? {
            0 => Packet::Goodbye,
            1 => Packet::SaveDelta(save::Delta::read(tcp_stream).await?),
            2 => Packet::SaveInit(save::Save::read(tcp_stream).await?),
            3 => Packet::KnowledgeInit(knowledge::Knowledge::read(tcp_stream).await?),
            n => return Err(PacketReadError::UnknownPacketId(n)),
        })
    }

    async fn write(&self, tcp_stream: &mut TcpStream) -> io::Result<()> {
        match self {
            Packet::Goodbye => 0u8.write(tcp_stream).await?,
            Packet::SaveDelta(delta) => {
                1u8.write(tcp_stream).await?;
                delta.write(tcp_stream).await?;
            }
            Packet::SaveInit(save) => {
                2u8.write(tcp_stream).await?;
                save.write(tcp_stream).await?;
            }
            Packet::KnowledgeInit(knowledge) => {
                3u8.write(tcp_stream).await?;
                knowledge.write(tcp_stream).await?;
            }
        }
        Ok(())
    }

    fn write_sync(&self, tcp_stream: &mut std::net::TcpStream) -> io::Result<()> {
        match self {
            Packet::Goodbye => 0u8.write_sync(tcp_stream)?,
            Packet::SaveDelta(delta) => {
                1u8.write_sync(tcp_stream)?;
                delta.write_sync(tcp_stream)?;
            }
            Packet::SaveInit(save) => {
                2u8.write_sync(tcp_stream)?;
                save.write_sync(tcp_stream)?;
            }
            Packet::KnowledgeInit(knowledge) => {
                3u8.write_sync(tcp_stream)?;
                knowledge.write_sync(tcp_stream)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, From, Clone)]
pub enum ReadError {
    Io(Arc<io::Error>),
    #[from]
    Packet(PacketReadError),
    VersionMismatch {
        server: u8,
        client: u8,
    },
}

impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> ReadError {
        ReadError::Io(Arc::new(e))
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::Io(e) => write!(f, "I/O error: {}", e),
            ReadError::Packet(e) => e.fmt(f),
            ReadError::VersionMismatch { server, client } => match client.cmp(server) {
                Less => write!(f, "An outdated auto-tracker attempted to connect. Please update the auto-tracker and try again."),
                Greater => write!(f, "An auto-tracker failed to connect because this app is outdated. Please update this app and try again."),
                Equal => unreachable!(),
            },
        }
    }
}

/// Reads packets from the given stream.
pub fn read(mut tcp_stream: TcpStream) -> impl Stream<Item = Result<Packet, ReadError>> {
    try_stream! {
        let version = u8::read(&mut tcp_stream).await?;
        if version != VERSION { Err(ReadError::VersionMismatch { server: VERSION, client: version })? }
        loop {
            let packet = Packet::read(&mut tcp_stream).await?;
            if let Packet::Goodbye = packet { break }
            yield packet
        }
    }
}

/// Writes the given packets to the given stream.
///
/// The handshake at the start and the `Goodbye` packet at the end are inserted automatically.
pub async fn write(mut tcp_stream: TcpStream, packets: impl Stream<Item = Packet>) -> io::Result<()> {
    VERSION.write(&mut tcp_stream).await?;
    pin_mut!(packets);
    while let Some(packet) = packets.next().await {
        packet.write(&mut tcp_stream).await?;
    }
    Packet::Goodbye.write(&mut tcp_stream).await?;
    Ok(())
}

pub fn handshake_sync(tcp_stream: &mut std::net::TcpStream) -> io::Result<()> {
    VERSION.write_sync(tcp_stream)?;
    Ok(())
}
