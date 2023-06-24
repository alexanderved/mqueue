use crate::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

/// Creates a new unidirectional message queue for dynamically typed messages.
///
/// A unidirectional queue is a queue where there is a flow of messages
/// which is directed only in one direction, from sender to receiver.
pub fn unidirectional_queue_dyn() -> (DynMessageSender, DynMessageReceiver) {
    let (send, recv) = mpsc::channel();
    let is_active = Arc::new(AtomicBool::new(true));

    let msg_send = DynMessageSender {
        send: send,
        is_active: Arc::clone(&is_active),
    };
    let msg_recv = DynMessageReceiver {
        recv: Arc::new(recv),
        is_active,
    };

    (msg_send, msg_recv)
}

/// The sending-half of the message queue for dynamically typed messages.
#[derive(Clone)]
pub struct DynMessageSender {
    send: Sender<Arc<dyn Message>>,
    pub(crate) is_active: Arc<AtomicBool>,
}

impl DynMessageSender {
    /// Returns the id of the queue which the [`DynMessageSender`] is part of.
    pub fn queue_id(&self) -> QueueId {
        // The pointer which is stored in `DynMessageSender::is_active` is unique
        // for each queue, so it can be used as id.
        QueueId::new(self.is_active.as_ref() as *const _ as usize)
    }

    /// Returns if the [`DynMessageSender`] is active.
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    /// Activates the [`DynMessageSender`].
    pub fn activate(&self) {
        self.is_active.store(true, Ordering::SeqCst);
    }

    /// Deactivates the [`DynMessageSender`].
    pub fn deactivate(&self) {
        self.is_active.store(false, Ordering::SeqCst);
    }

    /// Sends message to the queue if the [`DynMessageSender`] is active.
    pub fn send(&self, msg: Arc<dyn Message>) -> Result<(), MessagingError> {
        if !self.is_active() {
            return Err(MessagingError::QueueNotActive);
        }

        self.send
            .send(msg)
            .map_err(|_| MessagingError::MessageNotSent)
    }
}

/// The receiving-half of the message queue for dynamically typed messages.
#[derive(Clone)]
pub struct DynMessageReceiver {
    recv: Arc<Receiver<Arc<dyn Message>>>,
    pub(crate) is_active: Arc<AtomicBool>,
}

impl DynMessageReceiver {
    /// Returns the id of the queue which the [`DynMessageReceiver`] is part of.
    pub fn queue_id(&self) -> QueueId {
        // The pointer which is stored in `MessageReceiver::is_active` is unique
        // for each queue, so it can be used as id.
        QueueId::new(self.is_active.as_ref() as *const _ as usize)
    }

    /// Returns if the [`DynMessageReceiver`] is active.
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    /// Activates the [`DynMessageReceiver`].
    pub fn activate(&self) {
        self.is_active.store(true, Ordering::SeqCst);
    }

    /// Deactivates the [`DynMessageReceiver`].
    pub fn deactivate(&self) {
        self.is_active.store(false, Ordering::SeqCst);
    }

    /// Receives one message if there is any and the [`DynMessageReceiver`] is active.
    pub fn recv(&self) -> Option<Arc<dyn Message>> {
        if !self.is_active() {
            return None;
        }

        self.recv.try_recv().ok()
    }

    /// Returns an iterator which yields all pending messages.
    pub fn iter(&self) -> DynMessageIter<'_> {
        DynMessageIter { msg_recv: self }
    }
}

// Makes two dyn queues activate and deactivate at the same time.
pub(crate) fn synchronize_dyn_queues_activity(
    dst_queue: (&mut DynMessageSender, &mut DynMessageReceiver),
    src_queue: (&DynMessageSender, &DynMessageReceiver),
) {
    dst_queue.0.is_active = Arc::clone(&src_queue.0.is_active);
    dst_queue.1.is_active = Arc::clone(&src_queue.1.is_active);
}

/// Creates a new unidirectional message queue for statically typed messages.
///
/// A unidirectional queue is a queue where there is a flow of messages
/// which is directed only in one direction, from sender to receiver.
pub fn unidirectional_queue<M: Message>() -> (MessageSender<M>, MessageReceiver<M>) {
    let (send, recv) = mpsc::channel();
    let is_active = Arc::new(AtomicBool::new(true));

    let msg_send = MessageSender {
        send: send,
        is_active: Arc::clone(&is_active),
    };
    let msg_recv = MessageReceiver {
        recv: Arc::new(recv),
        is_active,
    };

    (msg_send, msg_recv)
}

/// The sending-half of the message queue for statically typed messages.
#[derive(Clone)]
pub struct MessageSender<M> {
    send: Sender<Arc<M>>,
    pub(crate) is_active: Arc<AtomicBool>,
}

impl<M: Message> MessageSender<M> {
    /// Returns the id of the queue which the [`MessageSender`] is part of.
    pub fn queue_id(&self) -> QueueId {
        // The pointer which is stored in `MessageSender::is_active` is unique
        // for each queue, so it can be used as id.
        QueueId::new(self.is_active.as_ref() as *const _ as usize)
    }

    /// Returns if the [`MessageSender`] is active.
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    /// Activates the [`MessageSender`].
    pub fn activate(&self) {
        self.is_active.store(true, Ordering::SeqCst);
    }

    /// Deactivates the [`MessageSender`].
    pub fn deactivate(&self) {
        self.is_active.store(false, Ordering::SeqCst);
    }

    /// Sends message to the queue if the [`MessageSender`] is active.
    pub fn send(&self, msg: Arc<M>) -> Result<(), MessagingError> {
        if !self.is_active() {
            return Err(MessagingError::QueueNotActive);
        }

        self.send
            .send(msg)
            .map_err(|_| MessagingError::MessageNotSent)
    }
}

/// The receiving-half of the message queue for statically typed messages.
#[derive(Clone)]
pub struct MessageReceiver<M> {
    recv: Arc<Receiver<Arc<M>>>,
    pub(crate) is_active: Arc<AtomicBool>,
}

impl<M: Message> MessageReceiver<M> {
    /// Returns the id of the queue which the [`MessageReceiver`] is part of.
    pub fn queue_id(&self) -> QueueId {
        // The pointer which is stored in `MessageReceiver::is_active` is unique
        // for each queue, so it can be used as id.
        QueueId::new(self.is_active.as_ref() as *const _ as usize)
    }

    /// Returns if the [`MessageReceiver`] is active.
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    /// Activates the [`MessageReceiver`].
    pub fn activate(&self) {
        self.is_active.store(true, Ordering::SeqCst);
    }

    /// Deactivates the [`MessageReceiver`].
    pub fn deactivate(&self) {
        self.is_active.store(false, Ordering::SeqCst);
    }

    /// Receives one message if there is any and the [`MessageReceiver`] is active.
    pub fn recv(&self) -> Option<Arc<M>> {
        if !self.is_active() {
            return None;
        }

        self.recv.try_recv().ok()
    }

    /// Returns an iterator which yields all pending messages.
    pub fn iter(&self) -> MessageIter<'_, M> {
        MessageIter { msg_recv: self }
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
