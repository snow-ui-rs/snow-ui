use crate::traits::{Message, MessageContext, MessageHandler};

/// An async-capable event bus used by examples to send and subscribe to typed messages.
/// This implementation is thread-safe using `Arc<Mutex>`.
pub struct EventBus {
    inner: std::sync::Mutex<
        std::collections::HashMap<
            std::any::TypeId,
            Vec<
                futures::channel::mpsc::UnboundedSender<
                    std::sync::Arc<dyn std::any::Any + Send + Sync>,
                >,
            >,
        >,
    >,
    // Registered handlers keyed by message TypeId
    handlers:
        std::sync::Mutex<std::collections::HashMap<std::any::TypeId, Vec<Box<dyn ErasedHandler>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(std::collections::HashMap::new()),
            handlers: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Send a typed message to all subscribers (synchronous in this API) and invoke any
    /// registered `MessageHandler<T>` implementations immediately (runs their `async`
    /// handlers to completion synchronously on the current thread).
    pub fn send<T: Message + Send + Sync>(&self, msg: T) {
        let arc = std::sync::Arc::new(msg) as std::sync::Arc<dyn std::any::Any + Send + Sync>;
        // first deliver to classic subscribers
        let guard = self.inner.lock().unwrap();
        if let Some(subs) = guard.get(&std::any::TypeId::of::<T>()) {
            for tx in subs.iter() {
                let _ = tx.unbounded_send(arc.clone());
            }
        }

        // then dispatch to registered handlers
        let mut ctx = MessageContext::default();
        let handlers_guard = self.handlers.lock().unwrap();
        if let Some(handlers) = handlers_guard.get(&std::any::TypeId::of::<T>()) {
            for h in handlers.iter() {
                h.handle_any(arc.clone(), &mut ctx);
            }
        }
    }

    /// Subscribe to messages of type `T`.
    /// Returns a receiver which yields notifications when messages of that type arrive.
    pub fn subscribe<T: Message + Send + Sync>(&self) -> EventBusReceiver<T> {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let mut guard = self.inner.lock().unwrap();
        guard
            .entry(std::any::TypeId::of::<T>())
            .or_default()
            .push(tx);
        EventBusReceiver {
            rx,
            _marker: std::marker::PhantomData,
        }
    }

    /// Register a handler instance (wrapped in `Arc<Mutex<_>>`) that implements
    /// `MessageHandler<T>` so it will be invoked when messages of type `T` are sent.
    pub fn register_handler<H, T>(&self, handler: std::sync::Arc<std::sync::Mutex<H>>)
    where
        H: MessageHandler<T> + 'static + Send + Sync,
        T: Message + 'static + Send + Sync,
    {
        let mut guard = self.handlers.lock().unwrap();
        guard
            .entry(std::any::TypeId::of::<T>())
            .or_default()
            .push(Box::new(HandlerBox::<H, T> {
                h: handler,
                _marker: std::marker::PhantomData,
            }));
    }
}

/// Receiver wrapper that yields a notification when a message of type `T` is received.
pub struct EventBusReceiver<T> {
    rx: futures::channel::mpsc::UnboundedReceiver<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Message + Send + Sync> EventBusReceiver<T> {
    /// Wait for the next message of type `T`.
    /// Returns `Ok(())` when a message arrives, or `Err(())` if the sender side closed.
    pub async fn recv(&mut self) -> Result<(), ()> {
        use futures::StreamExt;
        while let Some(arc) = self.rx.next().await {
            let _ = arc;
            return Ok(());
        }
        Err(())
    }
}

/// Trait used to type-erase message handlers so we can store them in a single map.
trait ErasedHandler: Send + Sync {
    fn handle_any(
        &self,
        msg: std::sync::Arc<dyn std::any::Any + Send + Sync>,
        ctx: &mut MessageContext,
    );
}

/// A concrete wrapper that holds an `Arc<Mutex<H>>` where `H: MessageHandler<T>`.
struct HandlerBox<H, T>
where
    H: MessageHandler<T> + 'static + Send + Sync,
    T: Message + 'static + Send + Sync,
{
    h: std::sync::Arc<std::sync::Mutex<H>>,
    _marker: std::marker::PhantomData<T>,
}

impl<H, T> ErasedHandler for HandlerBox<H, T>
where
    H: MessageHandler<T> + 'static + Send + Sync,
    T: Message + 'static + Send + Sync,
{
    fn handle_any(
        &self,
        msg: std::sync::Arc<dyn std::any::Any + Send + Sync>,
        ctx: &mut MessageContext,
    ) {
        if let Some(m) = (&*msg).downcast_ref::<T>() {
            let mut h = self.h.lock().unwrap();
            futures::executor::block_on(h.handle(m, ctx));
        }
    }
}

static EVENT_BUS: std::sync::OnceLock<std::sync::Mutex<EventBus>> = std::sync::OnceLock::new();

fn get_event_bus() -> &'static std::sync::Mutex<EventBus> {
    EVENT_BUS.get_or_init(|| std::sync::Mutex::new(EventBus::new()))
}

/// A small `EventBus` handle that proxies into a global `EventBus` instance.
#[derive(Clone, Copy)]
pub struct EventBusHandle;

impl EventBusHandle {
    pub fn send<T: Message + Send + Sync>(&self, msg: T) {
        let guard = get_event_bus().lock().unwrap();
        guard.send(msg);
    }

    pub fn subscribe<T: Message + Send + Sync>(&self) -> EventBusReceiver<T> {
        let guard = get_event_bus().lock().unwrap();
        guard.subscribe::<T>()
    }

    /// Register a handler instance for messages of type `T` with the global event bus.
    /// The handler should be wrapped in `Arc<Mutex<_>>` since the bus stores an `Arc`.
    pub fn register_handler<H, T>(&self, h: std::sync::Arc<std::sync::Mutex<H>>)
    where
        H: MessageHandler<T> + 'static + Send + Sync,
        T: Message + 'static + Send + Sync,
    {
        let guard = get_event_bus().lock().unwrap();
        guard.register_handler::<H, T>(h)
    }
}

pub fn event_bus() -> EventBusHandle {
    EventBusHandle {}
}
