use crate::*;

use std::collections::HashMap;
use std::sync::Arc;

/// Creates a new bidirectional message queue.
///
/// A bidirectional queue is a queue where there are two flows of messages which are
/// directed in opposite directions. It lets two objects communicate with each other.
pub fn bidirectional_queue_dyn() -> (DynMessageEndpoint, DynMessageEndpoint) {
    let (send1, recv1) = unidirectional_queue_dyn();
    let (mut send2, mut recv2) = unidirectional_queue_dyn();

    synchronize_queues_activity((&mut send2, &mut recv2), (&send1, &recv1));

    let end1 = DynMessageEndpoint {
        input: recv1,
        output: send2,
    };
    let end2 = DynMessageEndpoint {
        input: recv2,
        output: send1,
    };

    (end1, end2)
}

/// The half of the bidirectional queue which can be used both for sending and receiving messages.
#[derive(Clone)]
pub struct DynMessageEndpoint {
    input: DynMessageReceiver,
    output: DynMessageSender,
}

impl DynMessageEndpoint {
    pub fn queue_id(&self) -> QueueId {
        // The pointer which is stored in `is_active` in both `MessageEndpoint::input`
        // and `MessageEndpoint::output` is unique for each queue, so it can be used as id.
        QueueId::new(self.input.is_active.as_ref() as *const _ as usize)
    }

    /// Converts the [`DynMessageEndpoint`] to a [`DynMessageSender`].
    pub fn as_sender(&self) -> &DynMessageSender {
        &self.output
    }

    /// Converts the [`DynMessageEndpoint`] to a [`DynMessageReceiver`].
    pub fn as_receiver(&self) -> &DynMessageReceiver {
        &self.input
    }

    /// Returns if the [`DynMessageEndpoint`] is active.
    pub fn is_active(&self) -> bool {
        self.as_sender().is_active() && self.as_receiver().is_active()
    }

    /// Activates the [`DynMessageEndpoint`].
    pub fn activate(&self) {
        self.input.activate();
        self.output.activate();
    }

    /// Deactivates the [`DynMessageEndpoint`].
    pub fn deactivate(&self) {
        self.input.deactivate();
        self.output.deactivate();
    }

    /// Sends message to the queue if the [`DynMessageEndpoint`] is active.
    pub fn send(&self, msg: Arc<dyn Message>) -> Result<(), MessagingError> {
        self.as_sender().send(msg)
    }

    /// Receives one message if there is any and the [`DynMessageEndpoint`] is active.
    pub fn recv(&self) -> Option<Arc<dyn Message>> {
        self.as_receiver().recv()
    }

    /// Returns an iterator which yields all pending messages.
    pub fn iter(&self) -> MessageIter<'_> {
        self.as_receiver().iter()
    }
}

/// A set of [`DynMessageEndpoint`]s which can be used to manipulate them simultaneously.
pub struct DynMessageEndpoints {
    map: HashMap<QueueId, DynMessageEndpoint>,
}

impl DynMessageEndpoints {
    /// Constructs a new [`DynMessageEndpoints`].
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Creates a new bidirectional message queue and stores one of its [`DynMessageEndpoint`]
    /// in the list. The other [`DynMessageEndpoint`] is returned to the caller.
    pub fn new_queue(&mut self) -> DynMessageEndpoint {
        let (end1, end2) = bidirectional_queue_dyn();
        self.map.insert(end1.queue_id(), end1);

        end2
    }

    /// Destroys one of the message queues stored in the [`DynMessageEndpoints`].
    pub fn destroy_queue(&mut self, end: DynMessageEndpoint) {
        self.map.remove(&end.queue_id());
    }

    /// Adds the given [`DynMessageEndpoint`] to the list.
    pub fn add_endpoint(&mut self, end: DynMessageEndpoint) {
        self.map.insert(end.queue_id(), end);
    }

    /// Removes the [`DynMessageEndpoint`] which correspondes to the given [`DynMessageEndpoint`]
    /// from the [`DynMessageEndpoints`].
    pub fn remove_endpoint(&mut self, end: DynMessageEndpoint) {
        self.map.remove(&end.queue_id());
    }

    /// Sends the given message to all stored [`DynMessageEndpoint`]s.
    pub fn send(&self, msg: Arc<dyn Message>) -> Result<(), MessagingError> {
        self.map
            .values()
            .try_for_each(|end| end.send(Arc::clone(&msg)))
    }

    /// Receives one message if there is any.
    pub fn recv(&self) -> Option<Arc<dyn Message>> {
        for end in self.map.values() {
            if let Some(msg) = end.recv() {
                return Some(msg);
            }
        }

        None
    }

    /// Returns an iterator which yields all pending messages from all [`DynMessageEndpoint`]s.
    pub fn iter(&self) -> AbstractMessageIter<impl MessageIterator + '_> {
        AbstractMessageIter {
            iter: self.map.values().flat_map(|end| end.iter()),
        }
    }
}
