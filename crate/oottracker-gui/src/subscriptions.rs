use {
    std::{
        any::TypeId,
        hash::{
            Hash as _,
            Hasher,
        },
        sync::Arc,
    },
    futures::{
        prelude::*,
        stream::BoxStream,
    },
    iced_futures::subscription::Recipe,
    oottracker::net::Connection,
    crate::Message,
};

pub(crate) struct Subscription {
    conn: Arc<dyn Connection>,
}

impl Subscription {
    pub(crate) fn new(conn: Arc<dyn Connection>) -> Subscription {
        Subscription { conn }
    }
}

impl<H: Hasher, I> Recipe<H, I> for Subscription {
    type Output = Message;

    fn hash(&self, state: &mut H) {
        TypeId::of::<Self>().hash(state);
        self.conn.hash().hash(state);
    }

    fn stream(self: Box<Self>, _: BoxStream<'_, I>) -> BoxStream<'_, Message> {
        Box::pin(
            self.conn.packet_stream()
                .map(|result| match result {
                    Ok(packet) => Message::Packet(packet),
                    Err(e) => Message::ConnectionError(e.into()),
                })
                .chain(stream::once(async { Message::ClientDisconnected }))
        )
    }
}
