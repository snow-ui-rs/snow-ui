// Minimal mock of an `snow_ui` crate used by the example in `main.rs`.
// Provides the types and items referenced by the example so the crate builds.

/// Helper macro to construct a `Vec<Object>` from a heterogeneous
/// list of items by calling `.into()` on each item.
///
/// Example:
/// ```rust
/// # use snow_ui::prelude::*;
/// let children = snow_ui::list![
///     Text { text: "hi", ..default() },
///     TextClock { format: "%H:%M:%S", ..default() },
/// ];
/// ```
#[macro_export]
macro_rules! __list_item {
    // Struct literal with explicit `.. rest` - leave it as-is
    ($ty:ident { $($fields:tt)* .. $rest:expr }) => {
        $ty { $($fields)* .. $rest }
    };
    // Struct literal without `..` - append defaults from `default()` helper
    ($ty:ident { $($fields:tt)* }) => {
        $ty { $($fields)* .. $crate::default() }
    };
    // Fallback: arbitrary expression (e.g., already an Object or `.into()`able)
    ($e:expr) => { $e };
} 

#[macro_export]
macro_rules! list {
    ($($e:expr),* $(,)?) => {
        vec![$($crate::__list_item!($e).into()),*]
    };
} 

#[macro_export]
macro_rules! __obj_expr {
    ($e:expr) => {
        $crate::__list_item!($e).into()
    };
} 

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
        ClickHandler,
        Girl,
        GirlActions,
        HAlign,
        HairColor,
        InnerMovement,
        InnerTicker,
        IntoObject,
        Message,
        MessageReceiver,
        Row,
        SkinColor,
        Text,
        TextClock,
        UpdateContext,
        VAlign,
        VIEWPORT_HEIGHT,
        VIEWPORT_WIDTH,
        Object,
        World,
        event_bus,
    };

    // Re-export the derive macros and the `snow` attribute helper so examples can `use snow_ui::prelude::*` and write
    // `#[derive(IntoObject)]`, `#[derive(Message)]`, `#[snow]` and `obj! { ... }` without importing `snow_ui_macros` explicitly.
    pub use snow_ui_macros::{IntoObject, Message, message, snow};

    // Bring convenient macros into the prelude by re-exporting the crate-level
    // implementations so `use snow_ui::prelude::*` brings them into scope.
    pub use crate::{obj, list};

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
#[derive(Debug)]
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
#[derive(Debug)]
pub struct Card {
    pub children: Vec<Object>,
}

impl Default for Card {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Row {
    pub children: Vec<Object>,
}

impl Default for Row {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Element {
    Text(Text),
    TextClock(TextClock),
    Button(Button),
}

#[allow(dead_code)]
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            inner: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }

    /// Send a typed message to all subscribers (synchronous in this API).
    pub fn send<T: Message>(&self, msg: T) {
        let rc = std::rc::Rc::new(msg) as std::rc::Rc<dyn std::any::Any>;
        let guard = self.inner.borrow();
        if let Some(subs) = guard.get(&std::any::TypeId::of::<T>()) {
            for tx in subs.iter() {
                let _ = tx.unbounded_send(rc.clone());
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
#[derive(Debug, Default)]
pub struct Girl {
    pub hair_color: HairColor,
    pub skin_color: SkinColor,
    pub body_type: BodyType,
    pub appearance: Appearance,
    pub every_morning: Vec<GirlActions>,
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
