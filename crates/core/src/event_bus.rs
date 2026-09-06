use crate::traits::{Message, MessageContext, MessageHandler};
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::{self, Sender};

type ErasedMessage = std::sync::Arc<dyn std::any::Any + Send + Sync>;
type HandlerFuture<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;

struct QueuedMessage {
    type_id: std::any::TypeId,
    message: ErasedMessage,
}

fn run_event_worker(
    receiver: mpsc::Receiver<QueuedMessage>,
    handlers: std::sync::Arc<
        std::sync::Mutex<std::collections::HashMap<std::any::TypeId, Vec<Box<dyn ErasedHandler>>>>,
    >,
) {
    std::thread::Builder::new()
        .name("snow-ui-event-worker".to_string())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to build event bus runtime");

            while let Ok(queued) = receiver.recv() {
                let handler_guard = handlers
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                runtime.block_on(async {
                    if let Some(registered) = handler_guard.get(&queued.type_id) {
                        for handler in registered {
                            handler.handle_any(queued.message.clone()).await;
                        }
                    }
                });
                crate::request_render_refresh_for_active_window();
            }
        })
        .expect("failed to start event bus worker");
}

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
    handlers: std::sync::Arc<
        std::sync::Mutex<std::collections::HashMap<std::any::TypeId, Vec<Box<dyn ErasedHandler>>>>,
    >,
    queue: Sender<QueuedMessage>,
}

impl EventBus {
    pub fn new() -> Self {
        let (queue, receiver) = mpsc::channel();
        let handlers = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
        run_event_worker(receiver, handlers.clone());
        Self {
            inner: std::sync::Mutex::new(std::collections::HashMap::new()),
            handlers,
            queue,
        }
    }

    /// Send a typed message synchronously to subscribers and enqueue registered handlers.
    /// Handler futures are executed asynchronously by the event bus worker.
    pub fn send<T: Message + Send + Sync>(&self, msg: T) {
        let arc = std::sync::Arc::new(msg) as std::sync::Arc<dyn std::any::Any + Send + Sync>;
        eprintln!(
            "[snow-ui::event_bus] send::<{}>() start",
            std::any::type_name::<T>()
        );
        // first deliver to classic subscribers
        let guard = self.inner.lock().unwrap();
        if let Some(subs) = guard.get(&std::any::TypeId::of::<T>()) {
            eprintln!(
                "[snow-ui::event_bus] send::<{}>() subscriber_count={}",
                std::any::type_name::<T>(),
                subs.len()
            );
            for tx in subs.iter() {
                let _ = tx.unbounded_send(arc.clone());
            }
        }

        let _ = self.queue.send(QueuedMessage {
            type_id: std::any::TypeId::of::<T>(),
            message: arc,
        });
        eprintln!(
            "[snow-ui::event_bus] send::<{}>() end",
            std::any::type_name::<T>()
        );
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
        eprintln!(
            "[snow-ui::event_bus] register_handler::<{}>()",
            std::any::type_name::<T>()
        );
        let mut guard = self
            .handlers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
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
    fn handle_any(&self, msg: ErasedMessage) -> HandlerFuture<'_>;
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
    fn handle_any(&self, msg: ErasedMessage) -> HandlerFuture<'_> {
        let handler = self.h.clone();
        Box::pin(async move {
            if let Some(message) = msg.downcast_ref::<T>() {
                let mut context = MessageContext::default();
                let mut handler = handler
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                handler.handle(message, &mut context).await;
            }
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct PingMsg {}

    impl Message for PingMsg {}

    #[derive(Default)]
    struct PingHandler {
        count: usize,
    }

    impl MessageHandler<PingMsg> for PingHandler {
        async fn handle(&mut self, _: &PingMsg, _: &mut MessageContext) {
            self.count += 1;
        }
    }

    #[derive(Clone, Debug)]
    struct SlowPingMsg;

    impl Message for SlowPingMsg {}

    #[derive(Default)]
    struct SlowPingHandler {
        completed: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    impl MessageHandler<SlowPingMsg> for SlowPingHandler {
        async fn handle(&mut self, _: &SlowPingMsg, _: &mut MessageContext) {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            self.completed
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    #[test]
    fn event_bus_routes_generic_messages_to_registered_handlers() {
        let handler = std::sync::Arc::new(std::sync::Mutex::new(PingHandler::default()));
        let handler_for_bus = handler.clone();
        event_bus().register_handler::<PingHandler, PingMsg>(handler_for_bus);
        event_bus().send(PingMsg {});

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        while handler.lock().unwrap().count == 0 && std::time::Instant::now() < deadline {
            std::thread::yield_now();
        }

        assert_eq!(handler.lock().unwrap().count, 1);
    }

    #[test]
    fn send_returns_before_async_handler_completes() {
        let completed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let handler = std::sync::Arc::new(std::sync::Mutex::new(SlowPingHandler {
            completed: completed.clone(),
        }));
        event_bus().register_handler::<SlowPingHandler, SlowPingMsg>(handler);

        let started = std::time::Instant::now();
        event_bus().send(SlowPingMsg);
        assert!(started.elapsed() < std::time::Duration::from_millis(40));

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        while !completed.load(std::sync::atomic::Ordering::SeqCst)
            && std::time::Instant::now() < deadline
        {
            std::thread::yield_now();
        }
        assert!(completed.load(std::sync::atomic::Ordering::SeqCst));
    }
}
