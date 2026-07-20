use cosmic_text::{
    render_decoration, Attrs, Buffer, Color as CosmicColor, Family, FontSystem, LegacyRenderer,
    Metrics, Renderer as TextRenderer, Shaping, SwashCache,
};
use linebender_resource_handle::Blob;
use vello::kurbo::{Affine, Rect, RoundedRect, Stroke};
use vello::peniko::{Color as VelloColor, Fill, ImageAlphaType, ImageData, ImageFormat};
use vello::util::RenderContext;
use vello::{AaConfig, RenderParams, Scene};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowBuilder;

struct PixelBuffer {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl PixelBuffer {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height * 4) as usize],
        }
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: CosmicColor) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as u32;
        let y = y as u32;
        if x >= self.width || y >= self.height {
            return;
        }
        let offset = ((y * self.width + x) * 4) as usize;
        let [r, g, b, a] = color.as_rgba();
        self.pixels[offset..offset + 4].copy_from_slice(&[r, g, b, a]);
    }
}

fn make_button_label_image(
    text: &str,
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
) -> ImageData {
    let metrics = Metrics::new(28.0, 36.0);
    let mut buffer = Buffer::new(font_system, metrics);
    let attrs = Attrs::new().family(Family::SansSerif);

    buffer.set_text(text, &attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(font_system, false);

    let runs: Vec<_> = buffer.layout_runs().collect();
    let min_line_top = runs.iter().map(|run| run.line_top).fold(0.0, f32::min);
    let max_line_bottom = runs
        .iter()
        .map(|run| run.line_top + run.line_height)
        .fold(0.0, f32::max);
    let text_height = (max_line_bottom - min_line_top).ceil() as u32;

    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    for run in &runs {
        for glyph in run.glyphs {
            let x_offset = glyph.font_size * glyph.x_offset;
            min_x = min_x.min(glyph.x + x_offset);
            max_x = max_x.max(glyph.x + x_offset + glyph.w);
        }
    }
    let text_width = if min_x.is_finite() && max_x.is_finite() {
        (max_x - min_x).ceil() as u32
    } else {
        0
    };

    let padding = 20;
    let width = text_width + padding * 2;
    let height = text_height + padding * 2;
    let mut pixel_buffer = PixelBuffer::new(width, height);

    let mut callback = |x: i32, y: i32, _w: u32, _h: u32, color: CosmicColor| {
        pixel_buffer.set_pixel(x, y, color);
    };

    let mut renderer = LegacyRenderer {
        font_system,
        cache: swash_cache,
        callback: &mut callback,
    };

    let y_offset = padding as f32 - min_line_top;
    let x_offset = padding as f32 - min_x;
    for run in &runs {
        for glyph in run.glyphs {
            let physical = glyph.physical((x_offset, y_offset), 1.0);
            TextRenderer::glyph(&mut renderer, physical, CosmicColor::rgb(255, 255, 255));
        }
        render_decoration(&mut renderer, run, CosmicColor::rgb(255, 255, 255));
    }

    ImageData {
        data: Blob::from(pixel_buffer.pixels),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width,
        height,
    }
}

fn build_scene(width: u32, height: u32, label: &ImageData) -> Scene {
    let mut scene = Scene::new();
    let background = Rect::new(0.0, 0.0, width as f64, height as f64);
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        VelloColor::from_rgb8(24, 28, 48),
        None,
        &background,
    );

    let button_rect = RoundedRect::new(120.0, 220.0, 680.0, 340.0, 24.0);
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        VelloColor::from_rgb8(55, 110, 255),
        None,
        &button_rect,
    );
    scene.stroke(
        &Stroke::new(3.0),
        Affine::IDENTITY,
        VelloColor::from_rgb8(255, 255, 255),
        None,
        &button_rect,
    );

    let image_x = (width.saturating_sub(label.width) / 2) as f64;
    let image_y = (height.saturating_sub(label.height) / 2) as f64;
    scene.draw_image(label, Affine::translate((image_x, image_y)));

    scene
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = WindowBuilder::new()
        .with_title("button")
        .with_inner_size(PhysicalSize::new(800, 600))
        .build(&event_loop)
        .expect("Failed to create window");
    let window = std::rc::Rc::new(window);

    let mut context = RenderContext::new();
    let surface_window = window.clone();
    let mut surface = pollster::block_on(context.create_surface(
        &*surface_window,
        800,
        600,
        wgpu::PresentMode::Fifo,
    ))
    .expect("Failed to create render surface");
    let device_id = surface.dev_id;
    let mut renderer = vello::Renderer::new(
        &context.devices[device_id].device,
        vello::RendererOptions::default(),
    )
    .expect("Failed to create Vello renderer");

    let mut font_system = FontSystem::new();
    let mut swash_cache = SwashCache::new();
    let label_image = make_button_label_image("Press me", &mut font_system, &mut swash_cache);
    let mut scene = build_scene(800, 600, &label_image);
    let mut params = RenderParams {
        base_color: VelloColor::from_rgb8(24, 28, 48),
        width: 800,
        height: 600,
        antialiasing_method: AaConfig::Area,
    };

    event_loop
        .run(move |event, event_loop_target| {
            event_loop_target.set_control_flow(ControlFlow::Wait);

            match event {
                Event::NewEvents(start_cause) => {
                    if matches!(start_cause, StartCause::Init) {
                        window.request_redraw();
                    }
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => event_loop_target.exit(),
                    WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                        context.resize_surface(&mut surface, size.width, size.height);
                        params.width = size.width;
                        params.height = size.height;
                        scene = build_scene(size.width, size.height, &label_image);
                        window.request_redraw();
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == ElementState::Pressed
                            && event.logical_key == Key::Named(NamedKey::Escape)
                        {
                            event_loop_target.exit();
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        let device_handle = &context.devices[device_id];
                        let device = &device_handle.device;
                        let queue = &device_handle.queue;
                        let surface_texture = match surface.surface.get_current_texture() {
                            Ok(surface_texture) => surface_texture,
                            Err(err) => {
                                eprintln!("Failed to acquire surface texture: {err}");
                                return;
                            }
                        };
                        let surface_view = surface_texture
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        renderer
                            .render_to_texture(device, queue, &scene, &surface.target_view, &params)
                            .expect("Failed to render scene");

                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("Vello Button Encoder"),
                            });

                        surface.blitter.copy(
                            device,
                            &mut encoder,
                            &surface.target_view,
                            &surface_view,
                        );

                        queue.submit(Some(encoder.finish()));
                        surface_texture.present();
                        event_loop_target.exit();
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        .expect("Event loop failed");
}
