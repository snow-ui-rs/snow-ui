// Central facade for the `snow-ui` core crate.
//
// All implementation details are now split into separate modules for maintainability.

pub mod backend;
pub mod elements;
pub mod event_bus;
pub mod form;
pub mod girl;
pub mod handler;
pub mod layout;
pub mod object;
pub mod server_api;
pub mod state;
pub mod traits;
pub mod types;

// Re-export the public API for ergonomic `snow_ui::...` usage.
pub use crate::backend::{
    SnowAction, SnowApp, SnowComponent, SnowComponentInstance, SnowMessage, SnowNode, SnowRuntime,
    SnowState, SnowUpdate, SnowView, SnowWorld,
};
pub use crate::backend::masonry_backend;
pub use crate::elements::{Button, Element, IntervalTimer, Switch, Text, TextClock, TextInput};
pub use crate::event_bus::{EventBus, EventBusHandle, EventBusReceiver, event_bus};
pub use crate::form::Form;
pub use crate::girl::{Appearance, BodyType, Girl, GirlActions, HairColor, SkinColor};
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

// Pulled in by the old-day convenient prelude and `register_handler!` macro flow.
pub use inventory;

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
        register_handlers_for_instance,
    };

    pub use super::inventory;
    pub use crate::actions;
    pub use crate::register_handler;
    pub use snow_ui_macros::{IntoObject, Message, element, message};
    pub use snow_ui_macros::{list, obj};

    pub fn default<T: Default>() -> T {
        T::default()
    }
}

fn run_masonry_window(world: World) {
    use masonry::core::{ErasedAction, WidgetId};
    use masonry::dpi::LogicalSize;
    use masonry::widgets::ButtonPress;
    use masonry_winit::app::{AppDriver, DriverCtx, EventLoop, NewWindow, WindowId};
    use masonry_winit::winit::window::Window;

    struct LaunchDriver {
        window_id: WindowId,
        adapter: crate::backend::masonry_backend::MasonryAdapter,
    }

    impl AppDriver for LaunchDriver {
        fn on_action(
            &mut self,
            window_id: WindowId,
            _ctx: &mut DriverCtx<'_, '_>,
            _widget_id: WidgetId,
            action: ErasedAction,
        ) {
            debug_assert_eq!(window_id, self.window_id, "unknown window");

            if action.is::<ButtonPress>() {
                if let Some(message) = self.adapter.dispatch_action(&action) {
                    self.adapter.handle_message(&message);
                    let _ = self.adapter.render();
                }
            }
        }
    }

    let mut adapter = crate::backend::masonry_backend::MasonryAdapter::new();
    adapter.set_world(world.clone().into());
    let main_widget = adapter.render_library_world(&world).erased();

    let window_size = LogicalSize::new(500.0, 300.0);
    let window_attributes = Window::default_attributes()
        .with_title("Snow UI")
        .with_resizable(true)
        .with_min_inner_size(window_size);

    let driver = LaunchDriver {
        window_id: WindowId::next(),
        adapter,
    };

    masonry_winit::app::run(
        EventLoop::with_user_event(),
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
    run_masonry_window(builder());
}

/// Launch a concrete library `World` immediately.
pub fn launch_world(world: World) {
    run_masonry_window(world);
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
