//! Native foundation for Snow UI, built on Taffy, winit, and wgpu.

mod button;
mod event;
mod layout;
mod paint;
mod refresh;
mod renderer;
mod runtime;
mod text_cache;
mod text_render;

pub use button::Button;
pub use event::{
    NativeEvent, NativeEventReceiver, NativeEventSender, NativeEventSink, PointerButton, WidgetId,
    WinitEventAdapter, native_event_channel, native_event_channel_with_capacity,
};
pub use layout::{LayoutNode, LayoutRect, LayoutTree, LayoutTreeError};
pub use paint::{
    Bevel, Border, Color, FillRect, PaintFrame, PaintNode, PaintScene, Rect, RoundedRectNode,
    Shadow, TextNode,
};
pub use refresh::{RefreshBatch, RefreshHandle};
pub use renderer::{RenderError, RenderViewport, WgpuRenderer};
pub use runtime::{
    NativeRefreshHandle, NativeSurface, NativeSurfaceError, NativeUserEvent, RedrawProxy,
};
pub use taffy;
pub use wgpu;
pub use winit;
