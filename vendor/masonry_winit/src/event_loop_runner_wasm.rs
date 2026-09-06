use masonry_core::core::{DefaultProperties, ErasedAction, NewWidget, Widget};
use masonry_core::peniko::Color;
use winit::event_loop::{EventLoop as WinitEventLoop, EventLoopProxy as WinitEventLoopProxy};
use winit::window::{Window as WindowHandle, WindowAttributes};

use crate::app::AppDriver;
use crate::app_driver::WindowId;

#[derive(Debug)]
pub enum MasonryUserEvent {
    AccessKit(winit::window::WindowId, accesskit_winit::WindowEvent),
    AsyncAction(WindowId, ErasedAction),
}

pub struct NewWindow {
    pub id: WindowId,
    pub attributes: WindowAttributes,
    pub root_widget: NewWidget<dyn Widget>,
    pub base_color: Color,
}

impl NewWindow {
    pub fn new(attributes: WindowAttributes, root_widget: NewWidget<dyn Widget + 'static>) -> Self {
        Self::new_with_id(WindowId::next(), attributes, root_widget)
    }

    pub fn new_with_id(
        id: WindowId,
        attributes: WindowAttributes,
        root_widget: NewWidget<dyn Widget + 'static>,
    ) -> Self {
        Self {
            id,
            attributes,
            root_widget,
            base_color: Color::BLACK,
        }
    }

    pub fn with_base_color(mut self, base_color: Color) -> Self {
        self.base_color = base_color;
        self
    }
}

pub struct Window;
#[derive(Debug)]
pub struct MasonryState;

pub type EventLoop = WinitEventLoop<MasonryUserEvent>;
pub type EventLoopBuilder = winit::event_loop::EventLoopBuilder<MasonryUserEvent>;
pub type EventLoopProxy = WinitEventLoopProxy<MasonryUserEvent>;

pub fn run(
    _new_windows: Vec<NewWindow>,
    _app_driver: impl AppDriver + 'static,
    _default_properties: DefaultProperties,
) -> Result<(), winit::error::EventLoopError> {
    Ok(())
}

pub fn run_with(
    _event_loop: EventLoop,
    _new_windows: Vec<NewWindow>,
    _app_driver: impl AppDriver + 'static,
    _default_properties: DefaultProperties,
) -> Result<(), winit::error::EventLoopError> {
    Ok(())
}

impl From<accesskit_winit::Event> for MasonryUserEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        Self::AccessKit(event.window_id, event.window_event)
    }
}

impl Window {
    pub fn handle(&self) -> &WindowHandle {
        panic!("masonry_winit WASM renderer is not implemented")
    }
}
