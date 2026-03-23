use crate::object::Object;

/// Marker trait for types usable as messages in the event bus.
/// Implemented by `#[derive(Message)]`.
///
/// Note: this crate targets a single-threaded environment, so `Message` does not
/// require `Send`/`Sync` — only `'static` is required for type-based storage.
#[allow(dead_code)]
pub trait Message: 'static {}

/// Context passed into `InnerMovement::update` allowing widgets to read timing information.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UpdateContext {
    pub time: std::time::Instant,
}

/// A trait for internal widgets that update over time.
#[allow(dead_code)]
pub trait InnerMovement {
    fn update(&mut self, ctx: &mut UpdateContext);
}

/// A trait for internal widgets that run an async ticker loop.
/// Implementors should perform periodic async work (e.g., with `tokio::time::interval`).
#[allow(dead_code)]
pub trait InnerTicker {
    async fn ticker(&mut self);
}

/// A trait for widgets that handle clicks.
#[allow(dead_code)]
pub trait ClickHandler {
    async fn on_click(&mut self);
}

/// A trait for widgets that subscribe to messages and register background tasks.
/// `register` is called with `&mut self` and may await messages and mutate the widget's state.
#[allow(dead_code)]
pub trait MessageReceiver {
    async fn register(&mut self);
}

/// A context passed to message handlers. Extend as needed.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct MessageContext {
    // placeholder for future fields (e.g., access to widget tree, event loop handle, etc.)
}

/// A trait for asynchronous handlers which react to messages of type `T`.
#[allow(dead_code)]
pub trait MessageHandler<T: Message> {
    async fn handle(&mut self, msg: &T, ctx: &mut MessageContext);
}

pub trait IntoObject {
    fn into_object(self) -> Object;
}
