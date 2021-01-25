use {
    std::{
        any::TypeId,
        hash::{
            Hash as _,
            Hasher,
        },
        io,
        net::Ipv6Addr,
    },
    futures::{
        prelude::*,
        stream::BoxStream,
    },
    iced_futures::subscription::Recipe,
    tokio::net::TcpListener,
    tokio_stream::wrappers::TcpListenerStream,
    oottracker::proto::{
        self,
        TCP_PORT,
    },
    crate::Message,
};

pub(crate) struct Subscription;

impl<H: Hasher, I> Recipe<H, I> for Subscription {
    type Output = Message;

    fn hash(&self, state: &mut H) {
        TypeId::of::<Self>().hash(state);
    }

    fn stream(self: Box<Self>, _: BoxStream<'_, I>) -> BoxStream<'_, Message> {
        Box::pin(
            stream::once(async { io::Result::Ok(TcpListenerStream::new(TcpListener::bind((Ipv6Addr::LOCALHOST, TCP_PORT)).await?)) })
                .try_flatten()
                .map_ok(|tcp_stream|
                    stream::once(async { Ok(Message::ClientConnected) })
                        .chain(proto::read(tcp_stream).map_ok(Message::Packet))
                        .chain(stream::once(async { Ok(Message::ClientDisconnected) }))
                )
                .try_flatten()
                .map(|result| match result {
                    Ok(msg) => msg,
                    Err(e) => Message::NetworkError(e),
                })
        )
    }
}
