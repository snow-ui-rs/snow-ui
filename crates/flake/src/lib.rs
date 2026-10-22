//! Native foundation for Snow UI, built on Taffy, winit, and wgpu.

mod event;
mod layout;
mod paint;
mod refresh;
mod renderer;
mod runtime;
mod text_cache;
mod text_render;

pub use event::{
    NativeEvent, NativeEventReceiver, NativeEventSender, NativeEventSink, PointerButton, WidgetId,
    WinitEventAdapter, native_event_channel,
};
pub use layout::{LayoutNode, LayoutRect, LayoutTree, LayoutTreeError};
pub use paint::{Color, FillRect, PaintFrame, PaintNode, PaintScene, Rect, TextNode};
pub use refresh::{RefreshBatch, RefreshHandle};
pub use renderer::{RenderError, RenderViewport, WgpuRenderer};
pub use runtime::{
    NativeRefreshHandle, NativeSurface, NativeSurfaceError, NativeUserEvent, RedrawProxy,
};
pub use taffy;
pub use wgpu;
pub use winit;
