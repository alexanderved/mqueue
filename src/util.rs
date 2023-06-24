use std::any::Any;
use std::sync::Arc;

/// An id of a queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueueId(usize);

impl QueueId {
    // Constructs a new [`QueueId`] from the given value.
    pub(crate) fn new(value: usize) -> Self {
        Self(value)
    }
}

/// Downcasts [`Arc<dyn Message>`] to the one of the given types and runs the code
/// which corresponds to it.
///
/// [`Arc<dyn Message>`]: crate::message::Message
#[macro_export]
macro_rules! match_message {
    ($msg:ident { $( $msg_tt:tt )* }) => {
        {
            match_message!(@arm $msg as $( $msg_tt )*);
        }
    };
    (@arm $msg:ident as $msg_ty:ty => $msg_handler:block $(,)?) => {
        if let Ok($msg) = ::std::sync::Arc::clone(&$msg).downcast::<$msg_ty>() {
            $msg_handler;
        }
    };
    (@arm $msg:ident as $msg_ty:ty => $msg_handler:expr $(,)?) => {
        match_message!(@arm $msg as $msg_ty => { $msg_handler; });
    };
    (@arm $msg:ident as $msg_ty:ty => $msg_handler:block, $( $msg_tt:tt )*) => {
        match_message!(@arm $msg as $msg_ty => $msg_handler);
        match_message!(@arm $msg as $( $msg_tt )*);
    };
    (@arm $msg:ident as $msg_ty:ty => $msg_handler:expr, $( $msg_tt:tt )*) => {
        match_message!(@arm $msg as $msg_ty => { $msg_handler; }, $( $msg_tt )*);
    };
}

/// An iterator that can be evaluated without performing any extra activity.
pub trait IteratorRun: Iterator + Sized {
    /// Runs an iterator.
    fn run(self) {
        self.for_each(|_| {});
    }
}

impl<I: Iterator> IteratorRun for I {}

#[doc(hidden)]
pub trait AsAnyArc: Send + Sync + 'static {
    #[doc(hidden)]
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl<T: Send + Sync + 'static> AsAnyArc for T {
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

#[doc(hidden)]
pub trait AsAnyRef: 'static {
    #[doc(hidden)]
    fn as_any_ref(&self) -> &dyn Any;
}

impl<T: 'static> AsAnyRef for T {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}
