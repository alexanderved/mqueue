use std::any::TypeId;
use std::sync::Arc;

use crate::*;

/// A type which is used for communicating between publishers and subscribers.
pub trait Message: util::AsAnyRef + util::AsAnyArc + Send + Sync + 'static {
    /// Returns the type id of the message.
    fn type_id(&self) -> MessageTypeId {
        MessageTypeId(self.as_any_ref().type_id())
    }
}

impl dyn Message {
    /// Attempts to downcast [`Arc<dyn Message>`] to a concrete type.
    ///
    /// [`Arc<dyn Message>`]: Message
    pub fn downcast<M: Message>(self: Arc<Self>) -> Result<Arc<M>, Arc<Self>> {
        let this = Arc::clone(&self);
        self.as_any_arc().downcast().map_err(|_| this)
    }
}

/// The type id of the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageTypeId(pub TypeId);

impl MessageTypeId {
    /// Gets the [`MessageTypeId`] of the given generic type.
    pub fn of<M: Message>() -> Self {
        Self(TypeId::of::<M>())
    }
}
