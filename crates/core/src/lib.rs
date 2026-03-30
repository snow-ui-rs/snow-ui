// Central facade for the `snow-ui` core crate.
//
// All implementation details are now split into separate modules for maintainability.

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

/// Launch the UI using a builder function that returns a `World`.
///
/// Example: `snow_ui::launch(world);` where `fn world() -> World { ... }`.
pub fn launch<F: FnOnce() -> World>(builder: F) {
    let world = builder();
    println!("Launching snow_ui with world:\n{:#?}", world);
}
