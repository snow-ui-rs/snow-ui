// Minimal mock of an `snow_ui` crate used by the example in `main.rs`.
// Provides the types and items referenced by the example so the crate builds.

// ============================================================================
// Inventory-based handler registration system
// ============================================================================

/// A handler registry entry collected at compile time via `inventory`.
/// Each entry knows how to register its handler for a specific (Element, Message) pair.
pub struct HandlerRegistryEntry {
    /// TypeId of the element type this handler is for
    pub element_type_id: fn() -> std::any::TypeId,
    /// Registers the handler onto the given `Rc<RefCell<dyn Any>>` element instance.
    /// Returns `true` if the element was the expected type and registration succeeded.
    pub register_fn: fn(&std::rc::Rc<std::cell::RefCell<dyn std::any::Any>>),
}

inventory::collect!(HandlerRegistryEntry);

/// Register all handlers for a given element instance using the inventory.
/// This is called from the generated `into_object()` method.
pub fn register_handlers_for_instance<T: 'static>(instance: &std::rc::Rc<std::cell::RefCell<T>>) {
    let target_type_id = std::any::TypeId::of::<T>();
    // Create an Rc<RefCell<dyn Any>> from the instance for type-erased registration
    let any_rc: std::rc::Rc<std::cell::RefCell<dyn std::any::Any>> = instance.clone();

    for entry in inventory::iter::<HandlerRegistryEntry> {
        if (entry.element_type_id)() == target_type_id {
            (entry.register_fn)(&any_rc);
        }
    }
}

/// Check if there are any registered handlers for a given element type.
pub fn has_registered_handlers<T: 'static>() -> bool {
    let target_type_id = std::any::TypeId::of::<T>();
    for entry in inventory::iter::<HandlerRegistryEntry> {
        if (entry.element_type_id)() == target_type_id {
            return true;
        }
    }
    false
}

/// Macro to register a `MessageHandler` implementation and automatically submit it to inventory.
///
/// Usage:
/// ```rust
/// use snow_ui::prelude::*;
/// use snow_ui::register_handler;
///
/// #[derive(Message)]
/// struct MyMessage;
/// struct MyElement;
///
/// register_handler!(
///     impl MessageHandler<MyMessage> for MyElement {
///         async fn handle(&mut self, msg: &MyMessage, ctx: &mut MessageContext) {
///             let _ = (msg, ctx);
///         }
///     }
/// );
/// ```
///
/// This is equivalent to writing the impl block directly, plus it registers the handler
/// so that when `MyElement::into_object()` is called, the handler is automatically
/// registered with the event bus.
#[macro_export]
macro_rules! register_handler {
    (
        impl MessageHandler<$msg_ty:ty> for $elem_ty:ty {
            $($impl_body:tt)*
        }
    ) => {
        // Generate the actual trait implementation
        impl $crate::MessageHandler<$msg_ty> for $elem_ty {
            $($impl_body)*
        }

        // Submit a registry entry to inventory
        $crate::inventory::submit! {
            $crate::HandlerRegistryEntry {
                element_type_id: || ::std::any::TypeId::of::<$elem_ty>(),
                register_fn: |any_rc: &::std::rc::Rc<::std::cell::RefCell<dyn ::std::any::Any>>| {
                    // Clone the Rc and try to create a concrete typed Rc
                    // We need to use unsafe to reinterpret the Rc since RefCell<dyn Any> can't be directly downcast
                    // Instead, we'll borrow and check the type, then register if it matches
                    let borrowed = any_rc.borrow();
                    if borrowed.is::<$elem_ty>() {
                        drop(borrowed);
                        // Safety: We've verified the type matches, and we're in a single-threaded context
                        // We create a new Rc pointing to the same allocation but with concrete type
                        let ptr = ::std::rc::Rc::as_ptr(any_rc) as *const ::std::cell::RefCell<$elem_ty>;
                        ::std::mem::forget(any_rc.clone()); // Increment refcount
                        let concrete_rc = unsafe { ::std::rc::Rc::from_raw(ptr) };
                        $crate::event_bus().register_handler::<$elem_ty, $msg_ty>(concrete_rc);
                    }
                },
            }
        }
    };
}

// Re-export inventory for use in the macro
pub use inventory;

// Bring back an `obj!` macro at the core crate level so it sits alongside `list!`.
// This forwarding macro simply delegates to the `proc-macro` implementation in the
// `snow_ui_macros` crate so the behavior remains unchanged.
#[macro_export]
macro_rules! obj {
    ($($t:tt)*) => {
        ::snow_ui_macros::obj!($($t)*)
    };
}

pub mod prelude {
    pub use super::{
        Appearance,
        Board,
        BodyType,
        // Click demo types
        Button,
        Card,
        Form,
        ClickHandler,
        Girl,
        GirlActions,
        HAlign,
        HairColor,
        HandlerRegistryEntry,
        InnerMovement,
        InnerTicker,
        IntoObject,
        Message,
        MessageContext,
        MessageHandler,
        MessageReceiver,
        Object,
        Row,
        SkinColor,
        State,
        Switch,
        Text,
        TextClock,
        TextInput,
        UpdateContext,
        VAlign,
        VIEWPORT_HEIGHT,
        VIEWPORT_WIDTH,
        World,
        event_bus,
        has_registered_handlers,
        register_handlers_for_instance,
    };

    // Re-export inventory so user code can use the register_handler! macro
    pub use super::inventory;

    // Re-export the derive macros and the `element` attribute helper so examples can `use snow_ui::prelude::*` and write
    // `#[derive(IntoObject)]`, `#[derive(Message)]`, `#[element]` and `obj! { ... }` without importing `snow_ui_macros` explicitly.
    pub use snow_ui_macros::{IntoObject, Message, element, message};

    // Bring convenient macros into the prelude by re-exporting the proc-macro
    // implementations from the `snow_ui_macros` crate so `use snow_ui::prelude::*` brings
    // them into scope without needing to depend on `snow_ui_macros` directly.
    pub use crate::register_handler;
    pub use snow_ui_macros::{list, obj};

    /// Helper to allow `..default()` shorthand in user code (like Bevy's prelude).
    ///
    /// Example: `Row { ..default() }`
    #[allow(dead_code)]
    pub fn default<T: Default>() -> T {
        T::default()
    }
}

/// Launch the UI using a builder function that returns a `World`.
///
/// Example: `snow_ui::launch(world);` where `fn world() -> World { ... }`.
pub fn launch<F: FnOnce() -> World>(builder: F) {
    let world = builder();
    println!("Launching snow_ui with world:\n{:#?}", world);
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct World {
    pub root: Object,
}

impl Default for World {
    fn default() -> Self {
        Self {
            root: Object::Board(Board::default()),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Board {
    pub width: Size,
    pub height: Size,
    pub h_align: HAlign,
    pub v_align: VAlign,
    pub children: Vec<Object>,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            width: VIEWPORT_WIDTH,
            height: VIEWPORT_HEIGHT,
            h_align: HAlign::Center,
            v_align: VAlign::Middle,
            children: vec![],
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Card {
    pub children: Vec<Object>,
}

impl Default for Card {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Row {
    pub children: Vec<Object>,
}

impl Default for Row {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Element {
    Text(Text),
    TextClock(TextClock),
    Button(Button),
    Form(Form),
    TextInput(TextInput),
    Switch(Switch),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Text {
    pub text: &'static str,
}

impl Default for Text {
    fn default() -> Self {
        Self { text: "" }
    }
}

impl From<Text> for Element {
    fn from(t: Text) -> Self {
        Element::Text(t)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextClock {
    pub format: &'static str,
}

impl From<TextClock> for Element {
    fn from(t: TextClock) -> Self {
        Element::TextClock(t)
    }
}

impl Default for TextClock {
    fn default() -> Self {
        Self { format: "" }
    }
}

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

/// Object-safe submit-handler wrapper so `Form` can accept both `fn(&Form)` **and**
/// `async fn(&Form)` (and closures) without requiring callers to box/wrap them.
///
/// The trait returns a boxed future so implementations for async functions can
/// simply forward their returned future; synchronous functions are wrapped with
/// a ready future.
#[allow(dead_code)]
pub trait SubmitHandler {
    fn call_box(&self, form: &Form) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'static>>;
}

// Blanket impl for async functions / closures that return a `Future`.
impl<F, Fut> SubmitHandler for F
where
    F: Fn(&Form) -> Fut + 'static,
    Fut: std::future::Future<Output = ()> + 'static,
{
    fn call_box(&self, form: &Form) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'static>> {
        Box::pin((self)(form))
    }
}

/// A trait for asynchronous handlers which react to messages of type `T`.
#[allow(dead_code)]
pub trait MessageHandler<T: Message> {
    async fn handle(&mut self, msg: &T, ctx: &mut MessageContext);
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Size {
    ViewportWidth,
    ViewportHeight,
}

// A simple clickable button widget used by the `click` example.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Button {
    pub text: &'static str,
}

impl Default for Button {
    fn default() -> Self {
        Self { text: "" }
    }
}

impl From<Button> for Element {
    fn from(b: Button) -> Self {
        // Represent a button as an `Element::Button` variant so it can be wrapped
        // by `Widget::Element` like other inline elements.
        Element::Button(b)
    }
}

impl IntoObject for Button {
    fn into_object(self) -> Object {
        Element::from(self).into()
    }
}

// Text input field (very small demo stub)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextInput {
    /// Optional label text shown next to the input field.
    pub label: &'static str,
    pub name: &'static str,
    pub r#type: &'static str,
    /// Optional maximum length for input. If `0` then no limit is applied.
    pub max_len: u32,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            label: "",
            name: "",
            r#type: "text",
            max_len: 0,
        }
    }
}

impl From<TextInput> for Element {
    fn from(t: TextInput) -> Self {
        Element::TextInput(t)
    }
}

impl IntoObject for TextInput {
    fn into_object(self) -> Object {
        Element::from(self).into()
    }
}

// A simple switch container that chooses one of several children to show.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Switch {
    pub children: Vec<Object>,
}

impl Default for Switch {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

impl From<Switch> for Element {
    fn from(s: Switch) -> Self {
        Element::Switch(s)
    }
}

impl IntoObject for Switch {
    fn into_object(self) -> Object {
        Element::from(self).into()
    }
}

// Form element: groups input fields and exposes simple submit/reset controls.
#[allow(dead_code)]
#[derive(Clone)]
pub struct Form {
    /// Handler invoked on submit. Accepts async functions/closures; the macro
    /// will box function items automatically so user code stays ergonomic.
    pub submit_handler: std::sync::Arc<dyn SubmitHandler>,
    pub submit_button: Button,
    pub reset_button: Button,
    pub children: Vec<Object>,
}

impl Default for Form {
    fn default() -> Self {
        Self {
            submit_handler: std::sync::Arc::new(|_form: &Form| Box::pin(async move { })),
            submit_button: Button::default(),
            reset_button: Button::default(),
            children: vec![],
        }
    }
}

impl std::fmt::Debug for Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Form")
            .field("submit_handler", &"<handler>")
            .field("submit_button", &self.submit_button)
            .field("reset_button", &self.reset_button)
            .field("children", &self.children)
            .finish()
    }
}

impl From<Form> for Element {
    fn from(f: Form) -> Self {
        Element::Form(f)
    }
}

impl IntoObject for Form {
    fn into_object(self) -> Object {
        Element::from(self).into()
    }
}

pub const VIEWPORT_WIDTH: Size = Size::ViewportWidth;
pub const VIEWPORT_HEIGHT: Size = Size::ViewportHeight;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
}

// Object system
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Object {
    Board(Board),
    Girl(Girl),
    Card(Card),
    Row(Row),
    Element(Element),
}

impl From<Board> for Object {
    fn from(b: Board) -> Self {
        Object::Board(b)
    }
}

impl From<Girl> for Object {
    fn from(g: Girl) -> Self {
        Object::Girl(g)
    }
}

impl From<Card> for Object {
    fn from(c: Card) -> Self {
        Object::Card(c)
    }
}

impl From<Row> for Object {
    fn from(r: Row) -> Self {
        Object::Row(r)
    }
}

impl From<Element> for Object {
    fn from(e: Element) -> Self {
        Object::Element(e)
    }
}

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

impl From<Text> for Object {
    fn from(t: Text) -> Self {
        // Convert Text -> Element (via `From<Text> for Element`) and wrap into Object::Element
        Object::Element(t.into())
    }
}

impl From<TextClock> for Object {
    fn from(t: TextClock) -> Self {
        // Convert TextClock -> Element and wrap into Object::Element
        Object::Element(t.into())
    }
}

impl From<u128> for Object {
    fn from(n: u128) -> Self {
        // Convert number to a textual representation for demonstration.
        let s = format!("{}", n);
        let leaked: &'static str = Box::leak(s.into_boxed_str());
        Text { text: leaked }.into()
    }
}

pub trait IntoObject {
    fn into_object(self) -> Object;
}

impl<T: IntoObject> From<T> for Object {
    fn from(t: T) -> Self {
        t.into_object()
    }
}

// Girl component
#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct Girl {
    pub hair_color: HairColor,
    pub skin_color: SkinColor,
    pub body_type: BodyType,
    pub appearance: Appearance,
    pub every_morning: Vec<GirlActions>,
}

// ============================================================================
// State<T> - simple reactive-esque container for component state
// Stored as `Rc<RefCell<T>>` so we can cheaply clone and share between
// component instances and background tasks/handlers.
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct State<T> {
    inner: std::rc::Rc<std::cell::RefCell<T>>,
}

impl<T> State<T> {
    /// Create a new state wrapping the given value.
    pub fn new(value: T) -> Self {
        Self {
            inner: std::rc::Rc::new(std::cell::RefCell::new(value)),
        }
    }

    /// Get a cloned copy of the inner value (requires `T: Clone`).
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.inner.borrow().clone()
    }

    /// Set the inner value.
    pub fn set(&self, value: T) {
        *self.inner.borrow_mut() = value;
    }

    /// Mutate the inner value via a closure.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut b = self.inner.borrow_mut();
        f(&mut *b);
    }

    /// Borrow the inner value immutably (returns a `Ref<T>`).
    pub fn borrow(&self) -> std::cell::Ref<'_, T> {
        self.inner.borrow()
    }

    /// Borrow the inner value mutably (returns a `RefMut<T>`).
    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

impl<T: Default> Default for State<T> {
    fn default() -> Self {
        State::new(T::default())
    }
}

// Allow converting `State<T>` into an `Object` when the inner `T` can be converted.
impl<T> From<State<T>> for Object
where
    T: Clone + Into<Object>,
{
    fn from(s: State<T>) -> Self {
        s.get().into()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum HairColor {
    Black,
    Brown,
    Blonde,
    Red,
}

impl Default for HairColor {
    fn default() -> Self {
        HairColor::Brown
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum SkinColor {
    Yellow,
    Light,
    Dark,
}

impl Default for SkinColor {
    fn default() -> Self {
        SkinColor::Light
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum BodyType {
    Slim,
    Average,
    Curvy,
}

impl Default for BodyType {
    fn default() -> Self {
        BodyType::Average
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Appearance {
    Beautiful,
    Cute,
    Plain,
}

impl Default for Appearance {
    fn default() -> Self {
        Appearance::Cute
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum GirlActions {
    SayHi,
    PrepareBreakfast,
}
