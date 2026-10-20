use std::collections::HashSet;
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};

use winit::dpi::PhysicalSize;
use winit::event_loop::{EventLoop, EventLoopClosed, EventLoopProxy};
use winit::window::Window;
use winit::window::WindowId;

use crate::{Color, PaintFrame, RenderError, RenderViewport, WgpuRenderer};

/// User event used to wake an application handler for a native redraw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeUserEvent {
    Redraw(WindowId),
}

/// Thread-safe event-loop handle for requesting redraws from EventBus handlers.
#[derive(Clone)]
pub struct RedrawProxy {
    proxy: EventLoopProxy<NativeUserEvent>,
    pending_windows: Arc<Mutex<HashSet<WindowId>>>,
}

impl RedrawProxy {
    pub fn new(event_loop: &EventLoop<NativeUserEvent>) -> Self {
        Self {
            proxy: event_loop.create_proxy(),
            pending_windows: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn request_redraw(
        &self,
        window_id: WindowId,
    ) -> Result<(), EventLoopClosed<NativeUserEvent>> {
        let should_send = self.lock_pending().insert(window_id);
        if !should_send {
            return Ok(());
        }

        if let Err(error) = self.proxy.send_event(NativeUserEvent::Redraw(window_id)) {
            self.lock_pending().remove(&window_id);
            return Err(error);
        }
        Ok(())
    }

    /// Clear the coalescing marker after handling `NativeUserEvent::Redraw`.
    pub fn redraw_dispatched(&self, window_id: WindowId) {
        self.lock_pending().remove(&window_id);
    }

    fn lock_pending(&self) -> MutexGuard<'_, HashSet<WindowId>> {
        self.pending_windows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Cloneable invalidation handle for EventBus handlers and other worker threads.
#[derive(Clone)]
pub struct NativeRefreshHandle {
    refresh: crate::RefreshHandle,
    redraw: RedrawProxy,
    window_id: WindowId,
}

impl NativeRefreshHandle {
    pub fn new(redraw: RedrawProxy, window_id: WindowId) -> Self {
        Self {
            refresh: crate::RefreshHandle::default(),
            redraw,
            window_id,
        }
    }

    pub fn invalidate_layout(
        &self,
        widget_id: crate::WidgetId,
    ) -> Result<(), EventLoopClosed<NativeUserEvent>> {
        self.refresh.invalidate_layout(widget_id);
        self.wake()
    }

    pub fn invalidate_layout_path(
        &self,
        path: &[crate::WidgetId],
    ) -> Result<(), EventLoopClosed<NativeUserEvent>> {
        self.refresh.invalidate_layout_path(path);
        self.wake()
    }

    pub fn invalidate_paint(
        &self,
        widget_id: crate::WidgetId,
    ) -> Result<(), EventLoopClosed<NativeUserEvent>> {
        self.refresh.invalidate_paint(widget_id);
        self.wake()
    }

    pub fn invalidate_all(&self) -> Result<(), EventLoopClosed<NativeUserEvent>> {
        self.refresh.invalidate_all();
        self.wake()
    }

    pub fn take_pending(&self) -> crate::RefreshBatch {
        self.refresh.take_pending()
    }

    pub fn window_id(&self) -> WindowId {
        self.window_id
    }

    fn wake(&self) -> Result<(), EventLoopClosed<NativeUserEvent>> {
        self.redraw.request_redraw(self.window_id)
    }
}

/// Failures that can occur while creating or using a native window surface.
#[derive(Debug)]
pub enum NativeSurfaceError {
    CreateSurface(String),
    NoCompatibleAdapter,
    RequestDevice(String),
    UnsupportedSurface,
    InvalidSurfaceSize,
    FullFrameRequired,
    Surface(wgpu::SurfaceError),
    Render(RenderError),
}

impl From<RenderError> for NativeSurfaceError {
    fn from(error: RenderError) -> Self {
        Self::Render(error)
    }
}

/// Owns the winit window's wgpu surface and a persistent render target.
///
/// The persistent target preserves pixels between surface acquisitions so local
/// damage frames can be rendered safely before the complete image is presented.
pub struct NativeSurface {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: Option<wgpu::SurfaceConfiguration>,
    renderer: WgpuRenderer,
    offscreen: Option<wgpu::Texture>,
    offscreen_view: Option<wgpu::TextureView>,
    initialized: bool,
}

impl NativeSurface {
    /// Create a wgpu surface for a window. Call after winit's `resumed` callback.
    pub async fn new(window: Arc<Window>) -> Result<Self, NativeSurfaceError> {
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Err(NativeSurfaceError::InvalidSurfaceSize);
        }

        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .map_err(|error| NativeSurfaceError::CreateSurface(error.to_string()))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(|_| NativeSurfaceError::NoCompatibleAdapter)?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("snow-ui-flake device"),
                ..Default::default()
            })
            .await
            .map_err(|error| NativeSurfaceError::RequestDevice(error.to_string()))?;
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or(NativeSurfaceError::UnsupportedSurface)?;
        surface.configure(&device, &config);
        let renderer = WgpuRenderer::new(&device, &queue, config.format);
        let (offscreen, offscreen_view) = create_offscreen(&device, &config);

        Ok(Self {
            window,
            surface,
            adapter,
            device,
            queue,
            config: Some(config),
            renderer,
            offscreen: Some(offscreen),
            offscreen_view: Some(offscreen_view),
            initialized: false,
        })
    }

    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    pub fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    pub fn physical_size(&self) -> Option<[u32; 2]> {
        self.config
            .as_ref()
            .map(|config| [config.width, config.height])
    }

    /// Reconfigure the surface and recreate its persistent target after resize.
    /// A zero-sized window suspends rendering until the next nonzero resize.
    pub fn resize(&mut self, size: PhysicalSize<u32>) -> Result<(), NativeSurfaceError> {
        if size.width == 0 || size.height == 0 {
            self.config = None;
            self.offscreen = None;
            self.offscreen_view = None;
            self.initialized = false;
            return Ok(());
        }

        let mut config = self
            .surface
            .get_default_config(&self.adapter, size.width, size.height)
            .ok_or(NativeSurfaceError::UnsupportedSurface)?;
        if let Some(previous) = &self.config {
            config.format = previous.format;
            config.present_mode = previous.present_mode;
            config.alpha_mode = previous.alpha_mode;
            config.desired_maximum_frame_latency = previous.desired_maximum_frame_latency;
            config.view_formats = previous.view_formats.clone();
        }
        self.surface.configure(&self.device, &config);
        let (offscreen, offscreen_view) = create_offscreen(&self.device, &config);
        self.config = Some(config);
        self.offscreen = Some(offscreen);
        self.offscreen_view = Some(offscreen_view);
        self.initialized = false;
        Ok(())
    }

    /// Render a paint frame to the persistent target, then present its full image.
    pub fn render_frame(
        &mut self,
        frame: &PaintFrame,
        background: Color,
    ) -> Result<(), NativeSurfaceError> {
        let Some(config) = &self.config else {
            return Err(NativeSurfaceError::InvalidSurfaceSize);
        };
        if !self.initialized && !frame.full {
            return Err(NativeSurfaceError::FullFrameRequired);
        }
        let viewport_size = [config.width, config.height];
        let scale_factor = self.window.scale_factor() as f32;
        let offscreen_view = self
            .offscreen_view
            .as_ref()
            .ok_or(NativeSurfaceError::InvalidSurfaceSize)?;
        let surface_texture = match self.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(error @ (wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated)) => {
                self.surface.configure(&self.device, config);
                self.window.request_redraw();
                return Err(NativeSurfaceError::Surface(error));
            }
            Err(error) => return Err(NativeSurfaceError::Surface(error)),
        };
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let paint_commands = self.renderer.encode_frame(
            &self.device,
            &self.queue,
            offscreen_view,
            frame,
            RenderViewport {
                physical_size: viewport_size,
                scale_factor,
            },
            background,
        )?;
        let present_commands =
            self.renderer
                .encode_present(&self.device, offscreen_view, &surface_view, background);
        self.queue.submit([paint_commands, present_commands]);
        surface_texture.present();
        self.initialized = true;
        Ok(())
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

fn create_offscreen(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("snow-ui-flake persistent paint target"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}
