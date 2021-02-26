use {
    std::{
        cmp::Ordering::*,
        fmt,
        io,
        sync::Arc,
    },
    async_proto::Protocol,
    async_stream::try_stream,
    futures::prelude::*,
    pin_utils::pin_mut,
    serde_json::Value as Json,
    tokio::net::TcpStream,
    wheel::FromArc,
    crate::{
        knowledge,
        ram::Ram,
        save,
        ui::TrackerCellId,
    },
};

pub const TCP_PORT: u16 = 24801;
pub const VERSION: u8 = 3;

#[derive(Debug, Clone, Protocol)]
pub enum Packet {
    Goodbye,
    SaveDelta(save::Delta),
    SaveInit(save::Save),
    KnowledgeInit(knowledge::Knowledge),
    RamInit(Ram),
    UpdateCell(TrackerCellId, Json),
}

#[derive(Debug, FromArc, Clone)]
pub enum ReadError {
    #[from_arc]
    Io(Arc<io::Error>),
    #[from_arc]
    Packet(Arc<PacketReadError>),
    VersionMismatch {
        server: u8,
        client: u8,
    },
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
