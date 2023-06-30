use crate::*;

use std::collections::HashMap;

/// Creates a new bidirectional message queue for messages.
///
/// A bidirectional queue is a queue where there are two flows of messages which are
/// directed in opposite directions. It lets two objects communicate with each other.
pub fn bidirectional_queue<A: Message, B: Message>(
) -> (MessageEndpoint<A, B>, MessageEndpoint<B, A>) {
    let (send1, recv1) = unidirectional_queue();
    let (mut send2, mut recv2) = unidirectional_queue();

    synchronize_queues_activity((&mut send2, &mut recv2), (&send1, &recv1));

    let end1 = MessageEndpoint {
        input: recv1,
        output: send2,
    };
    let end2 = MessageEndpoint {
        input: recv2,
        output: send1,
    };

    (end1, end2)
}

/// The half of the bidirectional queue which can be used both
/// for sending and receiving messages.
pub struct MessageEndpoint<In, Out> {
    input: MessageReceiver<In>,
    output: MessageSender<Out>,
}

impl<In: Message, Out: Message> MessageEndpoint<In, Out> {
    pub fn queue_id(&self) -> QueueId {
        // The pointer which is stored in `is_active` in both `MessageEndpoint::input`
        // and `MessageEndpoint::output` is unique for each queue, so it can be used as id.
        QueueId::new(self.input.is_active.as_ref() as *const _ as usize)
    }

    /// Converts the [`MessageEndpoint`] to a [`MessageSender`].
    pub fn as_sender(&self) -> &MessageSender<Out> {
        &self.output
    }

    /// Converts the [`MessageEndpoint`] to a [`MessageReceiver`].
    pub fn as_receiver(&self) -> &MessageReceiver<In> {
        &self.input
    }

    /// Returns if the [`MessageEndpoint`] is active.
    pub fn is_active(&self) -> bool {
        self.as_sender().is_active() && self.as_receiver().is_active()
    }

    /// Activates the [`MessageEndpoint`].
    pub fn activate(&self) {
        self.input.activate();
        self.output.activate();
    }

    /// Deactivates the [`MessageEndpoint`].
    pub fn deactivate(&self) {
        self.input.deactivate();
        self.output.deactivate();
    }

    /// Sends message to the queue if the [`MessageEndpoint`] is active.
    pub fn send(&self, msg: Out) -> Result<(), MessagingError> {
        self.as_sender().send(msg)
    }

    /// Receives one message if there is any and the [`MessageEndpoint`] is active.
    pub fn recv(&self) -> Option<In> {
        self.as_receiver().recv()
    }

    /// Returns an iterator which yields all pending messages.
    pub fn iter(&self) -> MessageIter<'_, In> {
        self.as_receiver().iter()
    }

    /// Receives one message and forwards it into another queue.
    pub fn forward_one<Any, N>(&self, next: MessageEndpoint<Any, N>) -> Result<(), MessagingError>
    where
        Any: Message,
        N: Message + From<In>,
    {
        self.as_receiver().forward_one(next.as_sender().clone())
    }

    /// Forwards all pending messages into another queue.
    pub fn forward<Any, N>(&self, next: MessageEndpoint<Any, N>)
    where
        Any: Message,
        N: Message + From<In>,
    {
        self.as_receiver().forward(next.as_sender().clone());
    }
}

impl<In, Out> Clone for MessageEndpoint<In, Out> {
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            output: self.output.clone(),
        }
    }
}

/// A set of [`MessageEndpoint`]s which can be used to manipulate them simultaneously.
pub struct MessageEndpoints<In, Out> {
    map: HashMap<QueueId, MessageEndpoint<In, Out>>,
}

impl<In: Message, Out: Message> MessageEndpoints<In, Out> {
    /// Constructs a new [`MessageEndpoints`].
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Creates a new bidirectional message queue and stores one of its [`MessageEndpoint`]
    /// in the list. The other [`MessageEndpoint`] is returned to the caller.
    pub fn new_queue(&mut self) -> MessageEndpoint<Out, In> {
        let (end1, end2) = bidirectional_queue();
        self.map.insert(end1.queue_id(), end1);

        end2
    }

    /// Destroys one of the message queues stored in the [`MessageEndpoints`].
    pub fn destroy_queue(&mut self, end: MessageEndpoint<In, Out>) {
        self.map.remove(&end.queue_id());
    }

    /// Adds the given [`MessageEndpoint`] to the list.
    pub fn add_endpoint(&mut self, end: MessageEndpoint<In, Out>) {
        self.map.insert(end.queue_id(), end);
    }

    /// Removes the [`MessageEndpoint`] which correspondes to the given [`MessageEndpoint`]
    /// from the [`MessageEndpoints`].
    pub fn remove_endpoint(&mut self, end: MessageEndpoint<In, Out>) {
        self.map.remove(&end.queue_id());
    }

    /// Receives one message if there is any.
    pub fn recv(&self) -> Option<In> {
        for end in self.map.values() {
            if let Some(msg) = end.recv() {
                return Some(msg);
            }
        }

        None
    }

    /// Returns an iterator which yields all pending messages from all [`MessageEndpoint`]s.
    pub fn iter(&self) -> AbstractMessageIter<impl Iterator + '_> {
        AbstractMessageIter {
            iter: self.map.values().flat_map(|end| end.iter()),
        }
    }
}

impl<In: Message, Out: Message + Clone> MessageEndpoints<In, Out> {
    /// Sends the given message to all stored [`MessageEndpoint`]s.
    pub fn send(&self, msg: Out) -> Result<(), MessagingError> {
        self.map
            .values()
            .try_for_each(|end| end.send(msg.clone()))
    }
}

impl<In, Out> Clone for MessageEndpoints<In, Out> {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
        }
    }
}
