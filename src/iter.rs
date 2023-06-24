use crate::*;

use std::sync::Arc;

/// An iterator which yields dynamically typed messages.
pub trait DynMessageIterator: Iterator<Item = Arc<dyn Message>> + IteratorRun {
    /// Takes a message handler and creates an iterator which
    /// calls that message handler on each received message
    fn handle<M, H>(self, f: impl IntoMessageHandler<M, Handler = H>) -> HandleDynMessage<Self, H>
    where
        Self: Sized,
        M: Message + ?Sized,
        H: MessageHandler,
    {
        HandleDynMessage {
            iter: self,
            h: f.into_message_handler(),
        }
    }
}

impl<I: Iterator<Item = Arc<dyn Message>>> DynMessageIterator for I {}

/// An iterator which yields messages from one [`DynMessageReceiver`].
///
/// This `struct` is created by [`DynMessageReceiver::iter`] or [`DynMessageEndpoint::iter`].
pub struct DynMessageIter<'r> {
    pub(crate) msg_recv: &'r DynMessageReceiver,
}

impl Iterator for DynMessageIter<'_> {
    type Item = Arc<dyn Message>;

    fn next(&mut self) -> Option<Self::Item> {
        self.msg_recv.recv()
    }
}

/// An any iterator which yields dynamically typed messages.
pub struct AbstractDynMessageIter<I> {
    pub(crate) iter: I,
}

impl<I: DynMessageIterator> Iterator for AbstractDynMessageIter<I> {
    type Item = Arc<dyn Message>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// A message iterator which handles dynamically typed messages with `f`
/// and then forwards this message to the next handler.
///
/// This `struct` is created by [`DynMessageIterator::handle`].
pub struct HandleDynMessage<I, H> {
    iter: I,
    h: H,
}

impl<I: DynMessageIterator, H: MessageHandler> Iterator for HandleDynMessage<I, H> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let msg = self.iter.next();
        if let Some(msg) = msg.clone() {
            let _ = self.h.call(msg);
        }

        msg
    }
}

/// An iterator which yields statically typed messages.
pub trait MessageIterator: Iterator<Item = Arc<Self::Message>> + IteratorRun {
    type Message: Message;

    /// Takes a message handler and creates an iterator which
    /// calls that message handler on each received message
    fn handle<M, F>(self, f: F) -> HandleMessage<Self, F>
    where
        Self: Sized,
        M: Message,
        F: FnMut(Arc<M>),
    {
        HandleMessage { iter: self, f: f }
    }
}

impl<M: Message, I: Iterator<Item = Arc<M>>> MessageIterator for I {
    type Message = M;
}

/// An iterator which yields messages from one [`MessageReceiver`].
///
/// This `struct` is created by [`MessageReceiver::iter`] or [`MessageEndpoint::iter`].
pub struct MessageIter<'r, M> {
    pub(crate) msg_recv: &'r MessageReceiver<M>,
}

impl<M: Message> Iterator for MessageIter<'_, M> {
    type Item = Arc<M>;

    fn next(&mut self) -> Option<Self::Item> {
        self.msg_recv.recv()
    }
}

/// An any iterator which yields statically typed messages.
pub struct AbstractMessageIter<I> {
    pub(crate) iter: I,
}

impl<I: MessageIterator> Iterator for AbstractMessageIter<I> {
    type Item = Arc<I::Message>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// A message iterator which handles statically typed messages with `f`
/// and then forwards this message to the next handler.
///
/// This `struct` is created by [`MessageIterator::handle`].
pub struct HandleMessage<I, F> {
    iter: I,
    f: F,
}

impl<I: MessageIterator, F: FnMut(I::Item)> Iterator for HandleMessage<I, F> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let msg = self.iter.next();
        if let Some(msg) = msg.clone() {
            let _ = (self.f)(msg);
        }

        msg
    }
}
