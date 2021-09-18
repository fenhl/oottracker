use {
    std::{
        any::TypeId,
        hash::{
            Hash as _,
            Hasher,
        },
        marker::PhantomData,
        sync::Arc,
    },
    futures::{
        prelude::*,
        stream::BoxStream,
    },
    iced_futures::subscription::Recipe,
    ootr::Rando,
    oottracker::net::Connection,
    crate::Message,
};

pub(crate) struct Subscription<R: Rando> {
    conn: Arc<dyn Connection>,
    _rando: PhantomData<R>,
}

impl<R: Rando> Subscription<R> {
    pub(crate) fn new(conn: Arc<dyn Connection>) -> Subscription<R> {
        Subscription {
            conn,
            _rando: PhantomData,
        }
    }
}

impl<R: Rando, H: Hasher, I> Recipe<H, I> for Subscription<R> {
    type Output = Message<R>;

    fn hash(&self, state: &mut H) {
        TypeId::of::<Self>().hash(state);
        self.conn.hash().hash(state);
        TypeId::of::<R>().hash(state);
    }

    fn stream(self: Box<Self>, _: BoxStream<'_, I>) -> BoxStream<'_, Message<R>> {
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
