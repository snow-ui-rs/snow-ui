use crate::object::Object;

/// Marker trait for types usable as messages in the event bus.
/// Implemented by `#[derive(Message)]`.
///
/// Note: this crate targets a single-threaded environment, so `Message` does not
/// require `Send`/`Sync` — only `'static` is required for type-based storage.
pub trait Message: 'static + Send + Sync {}

/// Context passed into `InnerMovement::update` allowing widgets to read timing information.
#[derive(Debug, Clone)]
pub struct UpdateContext {
    pub time: std::time::Instant,
}

/// A trait for internal widgets that update over time.
pub trait InnerMovement {
    fn update(&mut self, ctx: &mut UpdateContext);
}

/// A trait for internal widgets that run an async ticker loop.
/// Implementors should perform periodic async work (e.g., with `tokio::time::interval`).
#[allow(async_fn_in_trait)]
pub trait InnerTicker {
    async fn ticker(&mut self);
}

/// A trait for widgets that handle clicks.
#[allow(async_fn_in_trait)]
pub trait ClickHandler {
    async fn on_click(&mut self);
}

/// A trait for widgets that subscribe to messages and register background tasks.
/// `register` is called with `&mut self` and may await messages and mutate the widget's state.
#[allow(async_fn_in_trait)]
pub trait MessageReceiver {
    async fn register(&mut self);
}

/// A context passed to message handlers. Extend as needed.
#[derive(Debug, Default)]
pub struct MessageContext {
    // placeholder for future fields (e.g., access to widget tree, event loop handle, etc.)
}

/// A trait for asynchronous handlers which react to messages of type `T`.
#[allow(async_fn_in_trait)]
pub trait MessageHandler<T: Message> {
    async fn handle(&mut self, msg: &T, ctx: &mut MessageContext);
}

pub trait IntoObject {
    fn into_object(self) -> Object;
}
