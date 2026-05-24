use glyphon::{Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport};
use pollster::block_on;
use std::sync::Arc;
use wgpu::{Backends, Color as WgpuColor, CompositeAlphaMode, CommandEncoderDescriptor, DeviceDescriptor, Instance, InstanceDescriptor, LoadOp, Operations, PowerPreference, PresentMode, RequestAdapterOptions, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor};
use winit::{dpi::PhysicalSize, event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowAttributes};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("DISPLAY").is_none()
        && std::env::var_os("WAYLAND_DISPLAY").is_none()
        && std::env::var_os("WAYLAND_SOCKET").is_none()
    {
        eprintln!("No graphical display detected. Please run this example in an X11 or Wayland session, or use a virtual framebuffer like xvfb-run.");
        std::process::exit(1);
    }

    let event_loop = EventLoop::new()?;
    #[allow(deprecated)]
    let window = Arc::new(
        event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("Glyphon Minimal Example")
                    .with_inner_size(PhysicalSize::new(800, 600)),
            )?,
    );
    let window_for_events = Arc::clone(&window);

    let mut state = block_on(State::new(&window));
    window.request_redraw();

    #[allow(deprecated)]
    event_loop
        .run(move |event, event_loop| {
            event_loop.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => event_loop.exit(),
                    WindowEvent::Resized(size) => {
                        state.resize(size.width, size.height);
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        let size = window_for_events.inner_size();
                        state.resize(size.width, size.height);
                    }
                    WindowEvent::RedrawRequested => {
                        state.render();
                    }
                    _ => {}
                },
                _ => {}
            }
        })?;

    Ok(())
}

struct State<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: SurfaceConfiguration,
    window_size: PhysicalSize<u32>,
    font_system: FontSystem,
    swash_cache: SwashCache,
    _cache: Cache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
    text_buffer: Buffer,
}

impl<'window> State<'window> {
    async fn new(window: &'window winit::window::Window) -> Self {
        let size = window.inner_size();

        let instance = Instance::new(InstanceDescriptor { backends: Backends::all(), ..InstanceDescriptor::new_without_display_handle() });
        let surface = instance.create_surface(window).expect("Failed to create surface");
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await
            .expect("Failed to create device");

        let surface_format = TextureFormat::Bgra8UnormSrgb;
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let _cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &_cache);
        let mut atlas = TextAtlas::new(&device, &queue, &_cache, surface_format);
        let text_renderer = TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);
        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(32.0, 44.0));

        let width = size.width as f32;
        let height = size.height as f32;
        text_buffer.set_size(&mut font_system, Some(width), Some(height));
        text_buffer.set_text(
            &mut font_system,
            "Hello glyphon!\nThis minimal example renders text into a wgpu window.",
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
            None,
        );
        text_buffer.shape_until_scroll(&mut font_system, false);

        let mut state = Self {
            surface,
            device,
            queue,
            config,
            window_size: size,
            font_system,
            swash_cache,
            _cache,
            viewport,
            atlas,
            text_renderer,
            text_buffer,
        };

        state.update_viewport();
        state
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.window_size = PhysicalSize::new(width, height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.update_viewport();
        }
    }

    fn update_viewport(&mut self) {
        self.viewport.update(
            &self.queue,
            Resolution {
                width: self.config.width,
                height: self.config.height,
            },
        );
    }

    fn render(&mut self) {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };

        let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("glyphon encoder"),
        });

        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [TextArea {
                    buffer: &self.text_buffer,
                    left: 20.0,
                    top: 20.0,
                    scale: 1.0,
                    bounds: TextBounds::default(),
                    default_color: Color::rgb(255, 255, 255),
                    custom_glyphs: &[],
                }],
                &mut self.swash_cache,
            )
            .expect("glyphon prepare failed");

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("glyphon pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(WgpuColor::BLACK),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.text_renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .expect("glyphon render failed");
        }

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}
