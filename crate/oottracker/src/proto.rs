use {
    std::{
        cmp::Ordering::*,
        fmt,
        pin::Pin,
    },
    async_proto::Protocol,
    async_stream::try_stream,
    derive_more::From,
    futures::{
        pin_mut,
        prelude::*,
    },
    serde_json::Value as Json,
    tokio::net::TcpStream,
    crate::{
        ModelDelta,
        ModelState,
        knowledge,
        ram::Ram,
        save,
        ui::TrackerCellId,
    },
};

pub const TCP_PORT: u16 = 24801;
pub const VERSION: u8 = 5;

#[derive(Debug, Clone, Protocol)]
pub enum Packet {
    Goodbye,
    SaveDelta(save::Delta),
    SaveInit(save::Save),
    KnowledgeInit(knowledge::Knowledge),
    RamInit(Ram),
    UpdateCell(TrackerCellId, Json),
    ModelInit(ModelState),
    ModelDelta(ModelDelta),
}

#[derive(Debug, From, Clone)]
pub enum ReadError {
    #[from]
    Packet(async_proto::ReadError),
    VersionMismatch {
        server: u8,
        client: u8,
    },
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::Packet(e) => e.fmt(f),
            ReadError::VersionMismatch { server, client } => match client.cmp(server) {
                Less => write!(f, "An outdated auto-tracking plugin attempted to connect. (Plugin uses protocol {} but this app uses protocol {}.) Please update the plugin and try again.", client, server),
                Greater => write!(f, "An auto-tracking plugin failed to connect because this app is outdated. (Plugin uses protocol {} but this app uses protocol {}.) Please update this app and try again.", client, server),
                Equal => unreachable!(),
            },
        }
    }
}

/// Reads packets from the given stream.
pub fn read(mut tcp_stream: TcpStream) -> Pin<Box<dyn Stream<Item = Result<Packet, ReadError>> + Send>> {
    Box::pin(try_stream! {
        let version = u8::read(&mut tcp_stream).await?;
        if version != VERSION { Err(ReadError::VersionMismatch { server: VERSION, client: version })? }
        loop {
            let packet = Packet::read(&mut tcp_stream).await?;
            if let Packet::Goodbye = packet { break }
            yield packet
        }
    })
}

/// Writes the given packets to the given stream.
///
/// The handshake at the start and the `Goodbye` packet at the end are inserted automatically.
pub async fn write(mut tcp_stream: TcpStream, packets: impl Stream<Item = Packet>) -> Result<(), async_proto::WriteError> {
    VERSION.write(&mut tcp_stream).await?;
    pin_mut!(packets);
    while let Some(packet) = packets.next().await {
        packet.write(&mut tcp_stream).await?;
    }
    Packet::Goodbye.write(&mut tcp_stream).await?;
    Ok(())
}

pub fn handshake_sync(tcp_stream: &mut std::net::TcpStream) -> Result<(), async_proto::WriteError> {
    VERSION.write_sync(tcp_stream)?;
    Ok(())
}
