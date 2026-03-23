// Central facade for the `snow-ui` core crate.
//
// All implementation details are now split into separate modules for maintainability.

pub mod types;
pub mod traits;
pub mod handler;
pub mod event_bus;
pub mod layout;
pub mod elements;
pub mod object;
pub mod form;
pub mod girl;
pub mod state;
pub mod server_api;

// Re-export the public API for ergonomic `snow_ui::...` usage.
pub use crate::types::{HAlign, Size, VAlign, VIEWPORT_HEIGHT, VIEWPORT_WIDTH};
pub use crate::traits::{
    ClickHandler,
    InnerMovement,
    InnerTicker,
    IntoObject,
    Message,
    MessageContext,
    MessageHandler,
    MessageReceiver,
    UpdateContext,
};
pub use crate::handler::{has_registered_handlers, register_handlers_for_instance, HandlerRegistryEntry};
pub use crate::event_bus::{event_bus, EventBus, EventBusHandle, EventBusReceiver};
pub use crate::layout::{Board, Card, Row};
pub use crate::elements::{Button, Element, IntervalTimer, Switch, Text, TextClock, TextInput};
pub use crate::object::{Object, World};
pub use crate::form::Form;
pub use crate::girl::{Appearance, BodyType, Girl, GirlActions, HairColor, SkinColor};
pub use crate::state::State;
pub use crate::server_api::ServerApi;

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
                register_fn: |any_rc: &::std::rc::Rc<::std::cell::RefCell<dyn ::std::any::Any>>| {
                    let borrowed = any_rc.borrow();
                    if borrowed.is::<$elem_ty>() {
                        drop(borrowed);
                        let ptr = ::std::rc::Rc::as_ptr(any_rc) as *const ::std::cell::RefCell<$elem_ty>;
                        ::std::mem::forget(any_rc.clone());
                        let concrete_rc = unsafe { ::std::rc::Rc::from_raw(ptr) };
                        $crate::event_bus().register_handler::<$elem_ty, $msg_ty>(concrete_rc);
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
        Appearance,
        Board,
        BodyType,
        Button,
        Card,
        ClickHandler,
        Form,
        Girl,
        GirlActions,
        HAlign,
        HairColor,
        HandlerRegistryEntry,
        InnerMovement,
        InnerTicker,
        IntervalTimer,
        IntoObject,
        Message,
        MessageContext,
        MessageHandler,
        MessageReceiver,
        Object,
        Row,
        ServerApi,
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

    pub use super::inventory;
    pub use snow_ui_macros::{IntoObject, Message, element, message};
    pub use crate::register_handler;
    pub use snow_ui_macros::{list, obj};
    pub use crate::actions;

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
