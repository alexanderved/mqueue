//! A simple implementation of message queues.

mod bidir_queue;
mod iter;
mod unidir_queue;

pub use bidir_queue::*;
pub use iter::*;
pub use unidir_queue::*;

/// A type which is used for communicating two objects.
pub trait Message: Send + 'static {}

impl<M: Message> Message for Box<M> {}
impl<M: Message + Sync> Message for std::sync::Arc<M> {}

#[derive(Debug)]
pub enum MessagingError {
    QueueClosed,
    QueueFull,
    QueueNotActive,
}

/// An id of a queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueueId(usize);

impl QueueId {
    // Constructs a new [`QueueId`] from the given value.
    pub(crate) fn new(value: usize) -> Self {
        Self(value)
    }
}
