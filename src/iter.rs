use crate::*;

/// An iterator that can be evaluated without performing any extra activity.
pub trait IteratorRun: Iterator + Sized {
    /// Runs an iterator.
    fn run(self) {
        self.for_each(|_| {});
    }
}

impl<I: Iterator> IteratorRun for I {}

/// An iterator which yields messages from one [`MessageReceiver`].
///
/// This `struct` is created by [`MessageReceiver::iter`] or [`MessageEndpoint::iter`].
pub struct MessageIter<'r, M> {
    pub(crate) msg_recv: &'r MessageReceiver<M>,
}

impl<M: Message> Iterator for MessageIter<'_, M> {
    type Item = M;

    fn next(&mut self) -> Option<Self::Item> {
        self.msg_recv.try_recv()
    }
}

/// An any iterator which yields messages.
pub struct AbstractMessageIter<I> {
    pub(crate) iter: I,
}

impl<M: Message, I: Iterator<Item = M>> Iterator for AbstractMessageIter<I> {
    type Item = M;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
