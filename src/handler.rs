use crate::*;

use std::marker::PhantomData;
use std::sync::Arc;

/// A type which handles messages.
pub trait MessageHandler {
    /// Handles the given message.
    fn call(&mut self, msg: Arc<dyn Message>) -> Result<(), MessagingError>;
}

/// A function for handling messages of specific type.
///
/// You can get it by calling [`IntoMessageHandler::into_message_handler`] on a function
/// which accepts a message.
pub struct MessageHandlerFunction<M: ?Sized, F> {
    f: F,
    _marker: PhantomData<fn(Arc<M>)>,
}

impl<M: Message, F: FnMut(Arc<M>)> MessageHandler for MessageHandlerFunction<M, F> {
    fn call(&mut self, msg: Arc<dyn Message>) -> Result<(), MessagingError> {
        (self.f)(
            msg.downcast()
                .map_err(|msg| MessagingError::WrongMessageType(msg))?,
        );

        Ok(())
    }
}

impl<F: FnMut(Arc<dyn Message>)> MessageHandler for MessageHandlerFunction<dyn Message, F> {
    fn call(&mut self, msg: Arc<dyn Message>) -> Result<(), MessagingError> {
        (self.f)(msg);

        Ok(())
    }
}

impl<H: MessageHandler + ?Sized> MessageHandler for &mut H {
    fn call(&mut self, msg: Arc<dyn Message>) -> Result<(), MessagingError> {
        (**self).call(msg)
    }
}

impl<H: MessageHandler + ?Sized> MessageHandler for Box<H> {
    fn call(&mut self, msg: Arc<dyn Message>) -> Result<(), MessagingError> {
        (**self).call(msg)
    }
}

/// A type which can be converted into a [`MessageHandler`].
pub trait IntoMessageHandler<M: Message + ?Sized>: Sized {
    type Handler: MessageHandler;

    /// Converts the type into the specific [`MessageHandler`].
    fn into_message_handler(self) -> Self::Handler;
}

impl<M: Message, F: FnMut(Arc<M>)> IntoMessageHandler<M> for F {
    type Handler = MessageHandlerFunction<M, F>;

    fn into_message_handler(self) -> Self::Handler {
        MessageHandlerFunction {
            f: self,
            _marker: PhantomData,
        }
    }
}

impl<F: FnMut(Arc<dyn Message>)> IntoMessageHandler<dyn Message> for F {
    type Handler = MessageHandlerFunction<dyn Message, F>;

    fn into_message_handler(self) -> Self::Handler {
        MessageHandlerFunction {
            f: self,
            _marker: PhantomData,
        }
    }
}

impl<M: Message, F: FnMut(Arc<M>)> IntoMessageHandler<M> for MessageHandlerFunction<M, F> {
    type Handler = Self;

    fn into_message_handler(self) -> Self::Handler {
        self
    }
}

impl<F: FnMut(Arc<dyn Message>)> IntoMessageHandler<dyn Message>
    for MessageHandlerFunction<dyn Message, F>
{
    type Handler = Self;

    fn into_message_handler(self) -> Self::Handler {
        self
    }
}

impl<M: Message + ?Sized> IntoMessageHandler<M> for &mut dyn MessageHandler {
    type Handler = Self;

    fn into_message_handler(self) -> Self::Handler {
        self
    }
}

impl<M: Message + ?Sized> IntoMessageHandler<M> for Box<dyn MessageHandler> {
    type Handler = Self;

    fn into_message_handler(self) -> Self::Handler {
        self
    }
}
