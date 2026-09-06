use std::fmt::Debug;
use std::hash::Hash;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};

use masonry_core::app::RenderRoot;
use masonry_core::core::{ErasedAction, WidgetId};
use tracing::field::DisplayValue;

use crate::app::{MasonryState, NewWindow, Window};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct WindowId(pub(crate) NonZeroU64);

impl WindowId {
    pub fn next() -> Self {
        static WINDOW_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = WINDOW_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(id.try_into().unwrap())
    }

    pub fn trace(self) -> DisplayValue<NonZeroU64> {
        tracing::field::display(self.0)
    }
}

#[derive(Debug)]
pub struct DriverCtx<'a> {
    state: &'a mut MasonryState,
}

impl<'a> DriverCtx<'a> {
    pub(crate) fn new(state: &'a mut MasonryState) -> Self {
        Self { state }
    }
}

#[derive(Debug)]
pub struct WgpuContext<'a> {
    pub instance: &'a wgpu::Instance,
    pub adapter: &'a wgpu::Adapter,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}

#[derive(Clone, Debug, Default)]
pub enum WgpuLimits {
    #[default]
    Default,
    Adapter,
    Custom(Box<wgpu::Limits>),
}

#[expect(unused_variables, reason = "Default impls do not use arguments")]
pub trait AppDriver {
    fn on_action(
        &mut self,
        window_id: WindowId,
        ctx: &mut DriverCtx<'_>,
        widget_id: WidgetId,
        action: ErasedAction,
    );

    fn on_async_action(
        &mut self,
        window_id: WindowId,
        ctx: &mut DriverCtx<'_>,
        action: ErasedAction,
    ) {
    }

    fn on_start(&mut self, state: &mut MasonryState) {}

    fn on_close_requested(&mut self, window_id: WindowId, ctx: &mut DriverCtx<'_>) {
        ctx.exit();
    }

    fn on_wgpu_ready(&mut self, _wgpu: &WgpuContext<'_>) {}
}

impl DriverCtx<'_> {
    pub fn render_root(&mut self, _window_id: WindowId) -> &mut RenderRoot {
        let _ = &mut self.state;
        panic!("masonry_winit WASM renderer is not implemented")
    }

    pub fn window(&mut self, _window_id: WindowId) -> &mut Window {
        let _ = &mut self.state;
        panic!("masonry_winit WASM renderer is not implemented")
    }

    pub fn create_window(&mut self, _new_window: NewWindow) {
        panic!("masonry_winit WASM renderer is not implemented")
    }

    pub fn close_window(&mut self, _window_id: WindowId) {
        panic!("masonry_winit WASM renderer is not implemented")
    }

    pub fn exit(&mut self) {
        let _ = &mut self.state;
    }
}
