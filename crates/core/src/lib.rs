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
#[derive(Debug)]
struct ClockRefresh;

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

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn request_clock_refresh_for_active_window() {
    let (Some(proxy), Some(window_id)) = (EVENT_LOOP_PROXY.get(), ACTIVE_WINDOW_ID.get()) else {
        return;
    };
    let _ = proxy.send_event(masonry_winit::app::MasonryUserEvent::AsyncAction(
        *window_id,
        Box::new(ClockRefresh),
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
    use masonry::core::{CollectionWidget, ErasedAction, WidgetId, WidgetTag};
    use masonry::dpi::LogicalSize;
    use masonry::widgets::Flex;
    use masonry::widgets::{ButtonPress, Label};
    use masonry_winit::app::{AppDriver, DriverCtx, EventLoop, NewWindow, WindowId};
    use masonry_winit::winit::window::Window;

    struct LaunchDriver {
        window_id: WindowId,
        world: World,
        native_tags: crate::object::NativeTags,
        switch_indices: Vec<usize>,
    }

    impl LaunchDriver {
        fn refresh_leaf_widgets(
            &self,
            window_id: WindowId,
            ctx: &mut DriverCtx<'_>,
            update_text: bool,
            update_buttons: bool,
            update_clocks: bool,
        ) {
            let mut text_values = Vec::new();
            let mut button_values = Vec::new();
            let mut clock_values = Vec::new();
            self.world.root.native_text_values(
                &mut text_values,
                &mut button_values,
                &mut clock_values,
            );
            if update_text {
                for (tag, value) in self.native_tags.text.iter().zip(text_values) {
                    ctx.render_root(window_id)
                        .edit_widget_with_tag(*tag, |mut label| {
                            Label::set_text(&mut label, value);
                        });
                }
            }
            if update_buttons {
                for (tag, value) in self.native_tags.button.iter().zip(button_values) {
                    ctx.render_root(window_id)
                        .edit_widget_with_tag(*tag, |mut label| {
                            Label::set_text(&mut label, value);
                        });
                }
            }
            if update_clocks {
                for (tag, value) in self.native_tags.clock.iter().zip(clock_values) {
                    ctx.render_root(window_id)
                        .edit_widget_with_tag(*tag, |mut label| {
                            Label::set_text(&mut label, value);
                        });
                }
            }
        }

        fn refresh_root(&mut self, window_id: WindowId, ctx: &mut DriverCtx<'_>) {
            let (text_count, button_count, clock_count) = self.world.root.native_tag_counts();
            self.native_tags = crate::object::NativeTags {
                text: (0..text_count)
                    .map(|_| WidgetTag::<Label>::unique())
                    .collect(),
                button: (0..button_count)
                    .map(|_| WidgetTag::<Label>::unique())
                    .collect(),
                clock: (0..clock_count)
                    .map(|_| WidgetTag::<Label>::unique())
                    .collect(),
                reset: (0..self.world.root.native_reset_count())
                    .map(|_| WidgetTag::unique())
                    .collect(),
                form: (0..self.world.root.native_form_count())
                    .map(|_| WidgetTag::unique())
                    .collect(),
            };
            let mut next_text = 0;
            let mut next_button = 0;
            let mut next_clock = 0;
            let mut next_reset = 0;
            let mut next_form = 0;
            let rebuilt = self
                .world
                .root
                .into_masonry_widget_with_native_tags(
                    &self.native_tags,
                    &mut next_text,
                    &mut next_button,
                    &mut next_clock,
                    &mut next_reset,
                    &mut next_form,
                )
                .erased();
            ctx.render_root(window_id).edit_layer(0, |mut root| {
                let mut flex = root.downcast::<Flex>();
                while flex.widget.len() > 0 {
                    Flex::remove(&mut flex, 0);
                }
                Flex::add_fixed(&mut flex, rebuilt);
            });
            self.switch_indices.clear();
            self.world
                .root
                .switch_active_indices(&mut self.switch_indices);
        }
    }

    impl AppDriver for LaunchDriver {
        fn on_action(
            &mut self,
            window_id: WindowId,
            ctx: &mut DriverCtx<'_>,
            widget_id: WidgetId,
            action: ErasedAction,
        ) {
            debug_assert_eq!(window_id, self.window_id, "unknown window");

            if action.is::<ButtonPress>() {
                let is_reset = self.native_tags.reset.iter().any(|tag| {
                    ctx.render_root(window_id)
                        .get_widget_with_tag(*tag)
                        .is_some_and(|widget| widget.id() == widget_id)
                });
                if is_reset {
                    let reset_index =
                        self.native_tags
                            .reset
                            .iter()
                            .enumerate()
                            .find_map(|(index, tag)| {
                                ctx.render_root(window_id)
                                    .get_widget_with_tag(*tag)
                                    .is_some_and(|widget| widget.id() == widget_id)
                                    .then_some(index)
                            });
                    if let Some(reset_index) = reset_index {
                        let form_index = reset_index;
                        let mut reset_index = reset_index;
                        if let Some(form) =
                            self.world.root.native_form_at_reset_index(&mut reset_index)
                        {
                            let mut target = form_index;
                            let mut next_text = 0;
                            let mut next_button = 0;
                            let mut next_clock = 0;
                            let mut next_reset = 0;
                            let offsets = self.world.root.native_form_tag_offsets(
                                &mut target,
                                &mut next_text,
                                &mut next_button,
                                &mut next_clock,
                                &mut next_reset,
                            );
                            if let Some((text_offset, button_offset, clock_offset, reset_offset)) =
                                offsets
                            {
                                let (text_count, button_count, clock_count) =
                                    form.native_tag_counts();
                                let local_tags = crate::object::NativeTags {
                                    text: (0..text_count)
                                        .map(|_| WidgetTag::<Label>::unique())
                                        .collect(),
                                    button: (0..button_count)
                                        .map(|_| WidgetTag::<Label>::unique())
                                        .collect(),
                                    clock: (0..clock_count)
                                        .map(|_| WidgetTag::<Label>::unique())
                                        .collect(),
                                    reset: (0..form.native_reset_count())
                                        .map(|_| WidgetTag::unique())
                                        .collect(),
                                    form: (0..form.native_form_count().saturating_sub(1))
                                        .map(|_| WidgetTag::unique())
                                        .collect(),
                                };
                                let mut local_text = 0;
                                let mut local_button = 0;
                                let mut local_clock = 0;
                                let mut local_reset = 0;
                                let mut local_form = 0;
                                if let Some(rebuilt) = form
                                    .into_masonry_form_contents_with_native_tags(
                                        &local_tags,
                                        &mut local_text,
                                        &mut local_button,
                                        &mut local_clock,
                                        &mut local_reset,
                                        &mut local_form,
                                    )
                                {
                                    let form_tag = self.native_tags.form[form_index];
                                    ctx.render_root(window_id).edit_widget_with_tag(
                                        form_tag,
                                        |mut form_widget| {
                                            while form_widget.widget.len() > 0 {
                                                Flex::remove(&mut form_widget, 0);
                                            }
                                            Flex::add_fixed(&mut form_widget, rebuilt);
                                        },
                                    );
                                    self.native_tags.text[text_offset..text_offset + text_count]
                                        .clone_from_slice(&local_tags.text);
                                    self.native_tags.button
                                        [button_offset..button_offset + button_count]
                                        .clone_from_slice(&local_tags.button);
                                    self.native_tags.clock
                                        [clock_offset..clock_offset + clock_count]
                                        .clone_from_slice(&local_tags.clock);
                                    self.native_tags.reset
                                        [reset_offset..reset_offset + local_tags.reset.len()]
                                        .clone_from_slice(&local_tags.reset);
                                }
                            }
                        }
                    }
                } else {
                    crate::trigger_clicks();
                }
            }
        }

        fn on_async_action(
            &mut self,
            window_id: WindowId,
            ctx: &mut DriverCtx<'_>,
            action: ErasedAction,
        ) {
            if action.is::<RenderRefresh>() {
                let mut current_switch_indices = Vec::new();
                self.world
                    .root
                    .switch_active_indices(&mut current_switch_indices);
                if current_switch_indices != self.switch_indices {
                    self.refresh_root(window_id, ctx);
                } else {
                    self.refresh_leaf_widgets(window_id, ctx, true, true, false);
                }
            } else if action.is::<ClockRefresh>() {
                self.refresh_leaf_widgets(window_id, ctx, false, false, true);
            }
        }
    }

    let (text_count, button_count, clock_count) = world.root.native_tag_counts();
    let native_tags = crate::object::NativeTags {
        text: (0..text_count)
            .map(|_| WidgetTag::<Label>::unique())
            .collect(),
        button: (0..button_count)
            .map(|_| WidgetTag::<Label>::unique())
            .collect(),
        clock: (0..clock_count)
            .map(|_| WidgetTag::<Label>::unique())
            .collect(),
        reset: (0..world.root.native_reset_count())
            .map(|_| WidgetTag::unique())
            .collect(),
        form: (0..world.root.native_form_count())
            .map(|_| WidgetTag::unique())
            .collect(),
    };
    let mut next_text = 0;
    let mut next_button = 0;
    let mut next_clock = 0;
    let mut next_reset = 0;
    let mut next_form = 0;
    let main_widget = world
        .root
        .into_masonry_widget_with_native_tags(
            &native_tags,
            &mut next_text,
            &mut next_button,
            &mut next_clock,
            &mut next_reset,
            &mut next_form,
        )
        .erased();
    let mut switch_indices = Vec::new();
    world.root.switch_active_indices(&mut switch_indices);

    let event_loop = EventLoop::with_user_event().build().unwrap();
    let _ = EVENT_LOOP_PROXY.set(event_loop.create_proxy());
    let _ = ACTIVE_WINDOW_ID.set(WindowId::next());
    if world.root.has_text_clock() {
        crate::runtime::interval(std::time::Duration::from_secs(1), || {
            request_clock_refresh_for_active_window();
        });
    }

    let window_size = LogicalSize::new(500.0, 300.0);
    let window_attributes = Window::default_attributes()
        .with_title("Snow UI")
        .with_resizable(true)
        .with_min_inner_size(window_size);

    let driver = LaunchDriver {
        window_id: *ACTIVE_WINDOW_ID.get().unwrap(),
        world: world.clone(),
        native_tags,
        switch_indices,
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
