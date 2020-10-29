use {
    std::{
        io,
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
pub const VERSION: u8 = 0;

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

#[derive(Debug, Clone)]
pub enum Packet {
    Goodbye,
    SaveDelta(save::Delta),
    SaveInit(save::Save),
    KnowledgeInit(knowledge::Knowledge),
}

#[derive(Debug, From, Clone)]
pub enum PacketReadError {
    #[from]
    DungeonRewardLocation(knowledge::DungeonRewardLocationReadError),
    Io(Arc<io::Error>),
    #[from]
    SaveData(save::SaveDataReadError),
    UnknownPacketId(u8),
}

impl From<io::Error> for PacketReadError {
    fn from(e: io::Error) -> PacketReadError {
        PacketReadError::Io(Arc::new(e))
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
