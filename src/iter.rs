use crate::*;

use std::sync::Arc;

/// An iterator which yields messages.
pub trait MessageIterator: Iterator<Item = Arc<dyn Message>> {
    /// Takes a message handler and creates an iterator which
    /// calls that message handler on each received message
    fn handle<'f, M, H>(self, f: impl IntoMessageHandler<M, Handler = H>) -> HandleMessage<'f, Self>
    where
        Self: Sized,
        M: Message + ?Sized,
        H: MessageHandler + 'f,
    {
        HandleMessage {
            iter: self,
            f: Box::new(f.into_message_handler()),
        }
    }

    /// Runs an iterator.
    fn run(self)
    where
        Self: Sized,
    {
        self.for_each(|_| {});
    }
}

impl<I: Iterator<Item = Arc<dyn Message>>> MessageIterator for I {}

/// An iterator which yields messages from one [`DynMessageReceiver`].
///
/// This `struct` is created by [`DynMessageReceiver::iter`] or [`MessageEndpoint::iter`].
pub struct MessageIter<'r> {
    pub(crate) msg_recv: &'r DynMessageReceiver,
}

impl Iterator for MessageIter<'_> {
    type Item = Arc<dyn Message>;

    fn next(&mut self) -> Option<Self::Item> {
        self.msg_recv.recv()
    }
}

pub struct AbstractMessageIter<I> {
    pub(crate) iter: I,
}

impl<I: MessageIterator> Iterator for AbstractMessageIter<I> {
    type Item = Arc<dyn Message>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// A message iterator which handles messages with `f`.
///
/// This `struct` is created by [`MessageIterator::handle`].
pub struct HandleMessage<'f, I> {
    iter: I,
    f: Box<dyn MessageHandler + 'f>,
}

impl<I: MessageIterator> Iterator for HandleMessage<'_, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let msg = self.iter.next();
        if let Some(msg) = msg.clone() {
            let _ = self.f.call(msg);
        }

        msg
    }
}
