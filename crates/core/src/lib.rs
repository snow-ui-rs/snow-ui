// Central facade for the `snow-ui` core crate.
//
// All implementation details are now split into separate modules for maintainability.

pub mod backend;
pub mod elements;
pub mod event_bus;
pub mod handler;
pub mod layout;
pub mod object;
pub mod runtime;
pub mod server_api;
pub mod state;
pub mod traits;
pub mod types;
#[cfg(target_arch = "wasm32")]
mod web;
pub mod widgets;

/// Platform-facing time types. The implementation can be replaced for WASM
/// without exposing the async runtime used by the core crate.
pub mod time {
    pub use std::time::Duration;
}

// Re-export the public API for ergonomic `snow_ui::...` usage.
#[cfg(not(target_arch = "wasm32"))]
pub use crate::backend::masonry_backend;
pub use crate::backend::{
    SnowAction, SnowApp, SnowComponent, SnowComponentInstance, SnowMessage, SnowNode, SnowRuntime,
    SnowState, SnowUpdate, SnowView, SnowWorld,
};
pub use crate::elements::{
    Appearance, BodyType, Button, Element, Form, Girl, GirlActions, HairColor, IntervalTimer,
    SkinColor, SubmitHandler, Switch, Text, TextClock, TextInput,
};
pub use crate::event_bus::{EventBus, EventBusHandle, EventBusReceiver, event_bus};
pub use crate::handler::{
    HandlerRegistryEntry, has_registered_handlers, register_handlers_for_instance,
};
pub use crate::layout::{Board, Card, Row};
pub use crate::object::{Object, World};
pub use crate::server_api::ServerApi;
pub use crate::state::State;
pub use crate::traits::{
    ClickHandler, InnerMovement, InnerTicker, IntoObject, Message, MessageContext, MessageHandler,
    MessageReceiver, UpdateContext,
};
pub use crate::types::{HAlign, Size, VAlign, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct RenderRefresh;

#[cfg(not(target_arch = "wasm32"))]
static EVENT_LOOP_PROXY: std::sync::OnceLock<masonry_winit::app::EventLoopProxy> =
    std::sync::OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
static ACTIVE_WINDOW_ID: std::sync::OnceLock<masonry_winit::app::WindowId> =
    std::sync::OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn request_render_refresh_for_active_window() {
    let (Some(proxy), Some(window_id)) = (EVENT_LOOP_PROXY.get(), ACTIVE_WINDOW_ID.get()) else {
        return;
    };
    let _ = proxy.send_event(masonry_winit::app::MasonryUserEvent::AsyncAction(
        *window_id,
        Box::new(RenderRefresh),
    ));
}

/// Run an async operation through Snow UI's runtime abstraction.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_async<F>(future: F)
where
    F: std::future::Future<Output = ()>,
{
    crate::runtime::run(future);
}

#[cfg(target_arch = "wasm32")]
pub fn run_async<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    crate::runtime::run(future);
}

// Pulled in by the old-day convenient prelude and `register_handler!` macro flow.
pub use inventory;

static CLICK_HANDLERS: std::sync::OnceLock<
    std::sync::Mutex<Vec<Box<dyn Fn() + Send + Sync + 'static>>>,
> = std::sync::OnceLock::new();

fn get_click_handlers() -> &'static std::sync::Mutex<Vec<Box<dyn Fn() + Send + Sync + 'static>>> {
    CLICK_HANDLERS.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

pub fn register_click_handler<F>(handler: F)
where
    F: Fn() + Send + Sync + 'static,
{
    eprintln!("[snow-ui] register_click_handler: registered one click callback");
    get_click_handlers().lock().unwrap().push(Box::new(handler));
}

pub fn trigger_clicks() {
    let handlers = get_click_handlers().lock().unwrap();
    eprintln!(
        "[snow-ui] trigger_clicks: {} handler(s) to run",
        handlers.len()
    );
    for handler in handlers.iter() {
        eprintln!("[snow-ui] trigger_clicks: invoking a click handler");
        handler();
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn trigger_click_handler(index: usize) {
    let handlers = get_click_handlers().lock().unwrap();
    if let Some(handler) = handlers.get(index) {
        handler();
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn reset_click_handlers() {
    get_click_handlers().lock().unwrap().clear();
}

/// Macro to register a `MessageHandler` implementation and automatically submit it to inventory.
///
/// Actual implementation is the same behavior that existed in the legacy monolithic `lib.rs`.
#[macro_export]
macro_rules! register_handler {
    (
        impl MessageHandler<$msg_ty:ty> for $elem_ty:ty {
            $($impl_body:tt)*
        }
    ) => {
        impl $crate::MessageHandler<$msg_ty> for $elem_ty {
            $($impl_body)*
        }

        $crate::inventory::submit! {
            $crate::HandlerRegistryEntry {
                element_type_id: || ::std::any::TypeId::of::<$elem_ty>(),
                register_fn: |any_arc: &::std::sync::Arc<::std::sync::Mutex<dyn ::std::any::Any + Send + Sync>>| {
                    let borrowed = any_arc.lock().unwrap();
                    if borrowed.is::<$elem_ty>() {
                        drop(borrowed);
                        let ptr = ::std::sync::Arc::as_ptr(any_arc) as *const ::std::sync::Mutex<$elem_ty>;
                        ::std::mem::forget(any_arc.clone());
                        let concrete_arc = unsafe { ::std::sync::Arc::from_raw(ptr) };
                        $crate::event_bus().register_handler::<$elem_ty, $msg_ty>(concrete_arc);
                    }
                },
            }
        }
    };
}

/// Forwarding `obj!` macro to the `snow_ui_macros` procedural macro implementation.
#[macro_export]
macro_rules! obj {
    ($($t:tt)*) => {
        ::snow_ui_macros::obj!($($t)*)
    };
}

/// Helper macro used by examples (`lovely_girl`) to build action vectors.
#[macro_export]
macro_rules! actions {
    ($($action:expr),* $(,)?) => {
        vec![$($action),*]
    };
}

pub mod prelude {
    pub use super::{
        Appearance, Board, BodyType, Button, Card, ClickHandler, Form, Girl, GirlActions, HAlign,
        HairColor, HandlerRegistryEntry, InnerMovement, InnerTicker, IntervalTimer, IntoObject,
        Message, MessageContext, MessageHandler, MessageReceiver, Object, Row, ServerApi,
        SkinColor, State, Switch, Text, TextClock, TextInput, UpdateContext, VAlign,
        VIEWPORT_HEIGHT, VIEWPORT_WIDTH, World, event_bus, has_registered_handlers,
        register_click_handler, register_handlers_for_instance, run_async, trigger_clicks,
    };

    pub use crate::time::Duration;

    pub use super::inventory;
    pub use crate::actions;
    pub use crate::register_handler;
    pub use snow_ui_macros::{IntoObject, Message, element, message};
    pub use snow_ui_macros::{list, obj};

    pub fn default<T: Default>() -> T {
        T::default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_masonry_window(world: World) {
    use masonry::core::{CollectionWidget, ErasedAction, WidgetId};
    use masonry::dpi::LogicalSize;
    use masonry::widgets::ButtonPress;
    use masonry_winit::app::{AppDriver, DriverCtx, EventLoop, NewWindow, WindowId};
    use masonry_winit::winit::window::Window;

    struct LaunchDriver {
        window_id: WindowId,
        world: World,
        adapter: crate::backend::masonry_backend::MasonryAdapter,
    }

    impl LaunchDriver {
        fn refresh_root(&self, window_id: WindowId, ctx: &mut DriverCtx<'_>) {
            let rebuilt_root = self.world.clone().into_masonry_widget();
            eprintln!("[snow-ui] LaunchDriver: rebuilding render root from live world");
            ctx.render_root(window_id).edit_layer(0, |mut root| {
                let mut flex = root.downcast::<masonry::widgets::Flex>();
                while flex.widget.len() > 0 {
                    masonry::widgets::Flex::remove(&mut flex, 0);
                }
                masonry::widgets::Flex::add_fixed(&mut flex, rebuilt_root);
            });
            eprintln!("[snow-ui] LaunchDriver: render root rebuild complete");
        }
    }

    impl AppDriver for LaunchDriver {
        fn on_action(
            &mut self,
            window_id: WindowId,
            ctx: &mut DriverCtx<'_>,
            _widget_id: WidgetId,
            action: ErasedAction,
        ) {
            debug_assert_eq!(window_id, self.window_id, "unknown window");

            if action.is::<ButtonPress>() {
                eprintln!("[snow-ui] AppDriver::on_action: ButtonPress received");
                crate::trigger_clicks();

                if let Some(message) = self.adapter.dispatch_action(&action) {
                    eprintln!(
                        "[snow-ui] AppDriver::on_action: dispatch_action produced message: {:?}",
                        message
                    );
                    self.adapter.handle_message(&message);
                    eprintln!("[snow-ui] AppDriver::on_action: handle_message completed");
                } else {
                    eprintln!("[snow-ui] AppDriver::on_action: dispatch_action returned None");
                }

                self.refresh_root(window_id, ctx);
            }
        }

        fn on_async_action(
            &mut self,
            window_id: WindowId,
            ctx: &mut DriverCtx<'_>,
            action: ErasedAction,
        ) {
            if action.is::<RenderRefresh>() {
                self.refresh_root(window_id, ctx);
            }
        }
    }

    let mut adapter = crate::backend::masonry_backend::MasonryAdapter::new();
    adapter.set_world(world.clone().into());
    let main_widget = adapter.render_library_world(&world).erased();

    let event_loop = EventLoop::with_user_event().build().unwrap();
    let _ = EVENT_LOOP_PROXY.set(event_loop.create_proxy());
    let _ = ACTIVE_WINDOW_ID.set(WindowId::next());
    crate::runtime::interval(std::time::Duration::from_secs(1), || {
        request_render_refresh_for_active_window();
    });

    let window_size = LogicalSize::new(500.0, 300.0);
    let window_attributes = Window::default_attributes()
        .with_title("Snow UI")
        .with_resizable(true)
        .with_min_inner_size(window_size);

    let driver = LaunchDriver {
        window_id: *ACTIVE_WINDOW_ID.get().unwrap(),
        world: world.clone(),
        adapter,
    };

    masonry_winit::app::run_with(
        event_loop,
        vec![NewWindow::new_with_id(
            driver.window_id,
            window_attributes,
            main_widget,
        )],
        driver,
        masonry::theme::default_property_set(),
    )
    .unwrap();
}

/// Launch the UI using a builder function that returns a `World`.
///
/// Example: `snow_ui::launch(world);` where `fn world() -> World { ... }`.
pub fn launch<F: FnOnce() -> World>(builder: F) {
    #[cfg(not(target_arch = "wasm32"))]
    run_masonry_window(builder());
    #[cfg(target_arch = "wasm32")]
    {
        crate::reset_click_handlers();
        crate::web::launch_world(builder());
    }
}

/// Launch a concrete library `World` immediately.
pub fn launch_world(world: World) {
    #[cfg(not(target_arch = "wasm32"))]
    run_masonry_window(world);
    #[cfg(target_arch = "wasm32")]
    crate::web::launch_world(world);
}

/// Launch a concrete `Object` as the app root.
pub fn launch_object(object: Object) {
    launch_world(World { root: object });
}

/// Launch a Snow component tree as the app root.
pub fn launch_component(component: SnowComponent) {
    launch_world(component.into_world());
}

/// Launch a Snow node tree as the app root.
pub fn launch_root(root: SnowNode) {
    launch_world(World { root: root.into() });
}
