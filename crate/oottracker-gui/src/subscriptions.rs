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

pub(crate) struct Subscription(pub(crate) Arc<dyn Connection>);

impl<H: Hasher, I> Recipe<H, I> for Subscription {
    type Output = Message;

    fn hash(&self, state: &mut H) {
        TypeId::of::<Self>().hash(state);
        self.0.hash().hash(state);
    }

    fn stream(self: Box<Self>, _: BoxStream<'_, I>) -> BoxStream<'_, Message> {
        Box::pin(
            self.0.packet_stream()
                .map(|result| match result {
                    Ok(packet) => Message::Packet(packet),
                    Err(e) => Message::ConnectionError(e.into()),
                })
                .chain(stream::once(async { Message::ClientDisconnected }))
        )
    }
}
