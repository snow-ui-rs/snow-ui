use crate::traits::{Message, MessageContext, MessageHandler};

/// An async-capable event bus used by examples to send and subscribe to typed messages.
/// This implementation is single-threaded: it uses `Rc` and `futures::channel::mpsc::unbounded` so
/// message types do not need to be `Send`/`Sync`.
#[allow(dead_code)]
pub struct EventBus {
    inner: std::cell::RefCell<
        std::collections::HashMap<
            std::any::TypeId,
            Vec<futures::channel::mpsc::UnboundedSender<std::rc::Rc<dyn std::any::Any>>>,
        >,
    >,
    // Registered handlers keyed by message TypeId
    handlers: std::cell::RefCell<
        std::collections::HashMap<std::any::TypeId, Vec<Box<dyn ErasedHandler>>>,
    >,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            inner: std::cell::RefCell::new(std::collections::HashMap::new()),
            handlers: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }

    /// Send a typed message to all subscribers (synchronous in this API) and invoke any
    /// registered `MessageHandler<T>` implementations immediately (runs their `async`
    /// handlers to completion synchronously on the current thread).
    pub fn send<T: Message>(&self, msg: T) {
        let rc = std::rc::Rc::new(msg) as std::rc::Rc<dyn std::any::Any>;
        // first deliver to classic subscribers
        let guard = self.inner.borrow();
        if let Some(subs) = guard.get(&std::any::TypeId::of::<T>()) {
            for tx in subs.iter() {
                let _ = tx.unbounded_send(rc.clone());
            }
        }

        // then dispatch to registered handlers
        let mut ctx = MessageContext::default();
        let handlers_guard = self.handlers.borrow();
        if let Some(handlers) = handlers_guard.get(&std::any::TypeId::of::<T>()) {
            for h in handlers.iter() {
                h.handle_any(rc.clone(), &mut ctx);
            }
        }
    }

    /// Subscribe to messages of type `T`.
    /// Returns a receiver which yields notifications when messages of that type arrive.
    pub fn subscribe<T: Message>(&self) -> EventBusReceiver<T> {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let mut guard = self.inner.borrow_mut();
        guard
            .entry(std::any::TypeId::of::<T>())
            .or_default()
            .push(tx);
        EventBusReceiver {
            rx,
            _marker: std::marker::PhantomData,
        }
    }

    /// Register a handler instance (wrapped in `Rc<RefCell<_>>`) that implements
    /// `MessageHandler<T>` so it will be invoked when messages of type `T` are sent.
    pub fn register_handler<H, T>(&self, handler: std::rc::Rc<std::cell::RefCell<H>>)
    where
        H: MessageHandler<T> + 'static,
        T: Message + 'static,
    {
        let mut guard = self.handlers.borrow_mut();
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
#[allow(dead_code)]
pub struct EventBusReceiver<T> {
    rx: futures::channel::mpsc::UnboundedReceiver<std::rc::Rc<dyn std::any::Any>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Message> EventBusReceiver<T> {
    /// Wait for the next message of type `T`.
    /// Returns `Ok(())` when a message arrives, or `Err(())` if the sender side closed.
    pub async fn recv(&mut self) -> Result<(), ()> {
        use futures::StreamExt;
        while let Some(rc) = self.rx.next().await {
            // We don't attempt to extract the value (no `downcast` required for this demo).
            // The presence of a message of the correct type is sufficient for example usage.
            let _ = rc;
            return Ok(());
        }
        Err(())
    }
}

/// Trait used to type-erase message handlers so we can store them in a single map.
trait ErasedHandler {
    fn handle_any(&self, msg: std::rc::Rc<dyn std::any::Any>, ctx: &mut MessageContext);
}

/// A concrete wrapper that holds an `Rc<RefCell<H>>` where `H: MessageHandler<T>`.
struct HandlerBox<H, T>
where
    H: MessageHandler<T> + 'static,
    T: Message + 'static,
{
    h: std::rc::Rc<std::cell::RefCell<H>>,
    _marker: std::marker::PhantomData<T>,
}

impl<H, T> ErasedHandler for HandlerBox<H, T>
where
    H: MessageHandler<T> + 'static,
    T: Message + 'static,
{
    fn handle_any(&self, msg: std::rc::Rc<dyn std::any::Any>, ctx: &mut MessageContext) {
        // Try to downcast to the concrete message type and call the async handler.
        if let Some(m) = (&*msg).downcast_ref::<T>() {
            let mut h = self.h.borrow_mut();
            // Run the async handler to completion on the current thread for now.
            futures::executor::block_on(h.handle(m, ctx));
        }
    }
}

thread_local! {
    static EVENT_BUS: std::cell::RefCell<EventBus> = std::cell::RefCell::new(EventBus::new());
}

/// A small `EventBus` handle that proxies into a thread-local `EventBus` instance.
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct EventBusHandle;

impl EventBusHandle {
    pub fn send<T: Message>(&self, msg: T) {
        EVENT_BUS.with(|b| b.borrow().send(msg));
    }

    pub fn subscribe<T: Message>(&self) -> EventBusReceiver<T> {
        EVENT_BUS.with(|b| b.borrow_mut().subscribe::<T>())
    }

    /// Register a handler instance for messages of type `T` with the global event bus.
    /// The handler should be wrapped in `Rc<RefCell<_>>` since the bus stores an `Rc`.
    pub fn register_handler<H, T>(&self, h: std::rc::Rc<std::cell::RefCell<H>>)
    where
        H: MessageHandler<T> + 'static,
        T: Message + 'static,
    {
        EVENT_BUS.with(|b| b.borrow_mut().register_handler::<H, T>(h))
    }
}

pub fn event_bus() -> EventBusHandle {
    EventBusHandle {}
}
