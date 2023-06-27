use crate::Message;

use std::fmt;
use std::sync::Arc;

pub enum MessagingError {
    WrongMessageType(Arc<dyn Message>),
    MessageNotSent,
    QueueNotActive,
}

impl fmt::Debug for MessagingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessagingError::WrongMessageType(_) => f.write_str("WrongMessageType"),
            MessagingError::MessageNotSent => f.write_str("MessageNotSent"),
            MessagingError::QueueNotActive => f.write_str("QueueNotActive"),
        }
    }
}
