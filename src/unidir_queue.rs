use crate::*;

use crossbeam_channel::{self, Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Creates a new unbounded unidirectional message queue.
///
/// A unidirectional queue is a queue where there is a flow of messages
/// which is directed only in one direction, from sender to receiver.
pub fn unidirectional_queue<M: Message>() -> (MessageSender<M>, MessageReceiver<M>) {
    let (send, recv) = crossbeam_channel::unbounded();
    let is_active = Arc::new(AtomicBool::new(true));

    let msg_send = MessageSender {
        send,
        is_active: Arc::clone(&is_active),
    };
    let msg_recv = MessageReceiver { recv, is_active };

    (msg_send, msg_recv)
}

/// Creates a new bounded unidirectional message queue.
///
/// A unidirectional queue is a queue where there is a flow of messages
/// which is directed only in one direction, from sender to receiver.
pub fn unidirectional_queue_bounded<M: Message>(
    cap: usize,
) -> (MessageSender<M>, MessageReceiver<M>) {
    let (send, recv) = crossbeam_channel::bounded(cap);
    let is_active = Arc::new(AtomicBool::new(true));

    let msg_send = MessageSender {
        send,
        is_active: Arc::clone(&is_active),
    };
    let msg_recv = MessageReceiver { recv, is_active };

    (msg_send, msg_recv)
}

/// The sending-half of the message queue for messages.
pub struct MessageSender<M> {
    send: Sender<M>,
    pub(crate) is_active: Arc<AtomicBool>,
}

impl<M: Message> MessageSender<M> {
    /// Returns the id of the queue which the [`MessageSender`] is part of.
    pub fn queue_id(&self) -> QueueId {
        // The pointer which is stored in `MessageSender::is_active` is unique
        // for each queue, so it can be used as id.
        QueueId::new(self.is_active.as_ref() as *const _ as usize)
    }

    /// Returns `true` if the message queue is empty.
    pub fn is_empty(&self) -> bool {
        self.send.is_empty()
    }

    /// Returns `true` if the message queue is full.
    pub fn is_full(&self) -> bool {
        self.send.is_full()
    }

    /// Returns `true` if the message queue is active.
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    /// Returns the number of messages in the queue.
    pub fn len(&self) -> usize {
        self.send.len()
    }

    /// Returns the capacity if the queue is bounded.
    pub fn capacity(&self) -> Option<usize> {
        self.send.capacity()
    }

    /// Activates the message queue.
    pub fn activate(&self) {
        self.is_active.store(true, Ordering::SeqCst);
    }

    /// Deactivates the message queue.
    pub fn deactivate(&self) {
        self.is_active.store(false, Ordering::SeqCst);
    }

    /// Sends message to the queue if the message queue is active.
    pub fn send(&self, msg: M) -> Result<(), MessagingError> {
        if !self.is_active() {
            return Err(MessagingError::QueueNotActive);
        }

        self.send.try_send(msg).map_err(|err| match err {
            crossbeam_channel::TrySendError::Disconnected(_) => MessagingError::QueueClosed,
            crossbeam_channel::TrySendError::Full(_) => MessagingError::QueueFull,
        })
    }
}

impl<M> Clone for MessageSender<M> {
    fn clone(&self) -> Self {
        Self {
            send: self.send.clone(),
            is_active: self.is_active.clone(),
        }
    }
}

/// The receiving-half of the message queue for messages.
pub struct MessageReceiver<M> {
    recv: Receiver<M>,
    pub(crate) is_active: Arc<AtomicBool>,
}

impl<M: Message> MessageReceiver<M> {
    /// Returns the id of the queue which the [`MessageReceiver`] is part of.
    pub fn queue_id(&self) -> QueueId {
        // The pointer which is stored in `MessageReceiver::is_active` is unique
        // for each queue, so it can be used as id.
        QueueId::new(self.is_active.as_ref() as *const _ as usize)
    }

    /// Returns `true` if the message queue is empty.
    pub fn is_empty(&self) -> bool {
        self.recv.is_empty()
    }

    /// Returns `true` if the message queue is full.
    pub fn is_full(&self) -> bool {
        self.recv.is_full()
    }

    /// Returns `true` if the message queue is active.
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    /// Returns the number of messages in the queue.
    pub fn len(&self) -> usize {
        self.recv.len()
    }

    /// Returns the capacity if the queue is bounded.
    pub fn capacity(&self) -> Option<usize> {
        self.recv.capacity()
    }

    /// Activates the message queue.
    pub fn activate(&self) {
        self.is_active.store(true, Ordering::SeqCst);
    }

    /// Deactivates the message queue.
    pub fn deactivate(&self) {
        self.is_active.store(false, Ordering::SeqCst);
    }

    /// Receives one message if there is any and the message queue is active.
    pub fn recv(&self) -> Option<M> {
        if !self.is_active() {
            return None;
        }

        self.recv.try_recv().ok()
    }

    /// Returns an iterator which yields all pending messages.
    pub fn iter(&self) -> MessageIter<'_, M> {
        MessageIter { msg_recv: self }
    }

    /// Receives one message and forwards it into another queue.
    pub fn forward_one<N>(&self, next: MessageSender<N>) -> Result<(), MessagingError>
    where
        N: Message + From<M>,
    {
        self.recv().map_or(Ok(()), |msg| next.send(msg.into()))
    }

    /// Forwards all pending messages into another queue.
    pub fn forward<N>(&self, next: MessageSender<N>)
    where
        N: Message + From<M>,
    {
        self.iter().for_each(|msg| {
            let _ = next.send(msg.into());
        });
    }
}

impl<M> Clone for MessageReceiver<M> {
    fn clone(&self) -> Self {
        Self {
            recv: self.recv.clone(),
            is_active: self.is_active.clone(),
        }
    }
}

// Makes two queues activate and deactivate at the same time.
pub(crate) fn synchronize_queues_activity<A, B>(
    dst_queue: (&mut MessageSender<A>, &mut MessageReceiver<A>),
    src_queue: (&MessageSender<B>, &MessageReceiver<B>),
) {
    dst_queue.0.is_active = Arc::clone(&src_queue.0.is_active);
    dst_queue.1.is_active = Arc::clone(&src_queue.1.is_active);
}
