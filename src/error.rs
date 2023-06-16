use crate::Message;

use std::sync::Arc;

pub enum MessagingError {
    WrongMessageType(Arc<dyn Message>),
    MessageNotSent,
    QueueNotActive,
}
