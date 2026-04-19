use std::rc::Rc;

use vello::kurbo::{Affine, Ellipse, Point, Rect, Stroke, Vec2};
use vello::peniko::{Color, Fill};
use vello::util::RenderContext;
use vello::{wgpu, AaConfig, RenderParams, Renderer, RendererOptions, Scene};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowBuilder;

fn build_scene(width: u32, height: u32) -> Scene {
    let mut scene = Scene::new();
    let background = Rect::new(0.0, 0.0, width as f64, height as f64);

    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgb8(20, 24, 38),
        None,
        &background,
    );

    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgb8(44, 121, 255),
        None,
        &Rect::new(80.0, 80.0, 320.0, 220.0),
    );

    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgb8(255, 235, 150),
        None,
        &Ellipse::new(Point::new(520.0, 320.0), Vec2::new(120.0, 120.0), 0.0),
    );

    scene.stroke(
        &Stroke::new(6.0),
        Affine::IDENTITY,
        Color::from_rgb8(255, 255, 255),
        None,
        &Rect::new(80.0, 80.0, 320.0, 220.0),
    );

    scene
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = WindowBuilder::new()
        .with_title("Vello Render Example")
        .with_inner_size(PhysicalSize::new(800, 600))
        .build(&event_loop)
        .expect("Failed to create window");
    let window = Rc::new(window);

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
    let mut renderer = Renderer::new(
        &context.devices[device_id].device,
        RendererOptions::default(),
    )
    .expect("Failed to create Vello renderer");

    let mut scene = build_scene(800, 600);
    let mut params = RenderParams {
        base_color: Color::from_rgb8(20, 24, 38),
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
                        scene = build_scene(size.width, size.height);
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
                            Err(error) => {
                                eprintln!("Failed to acquire surface texture: {error}");
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
                                label: Some("Vello Render Encoder"),
                            });

                        surface.blitter.copy(
                            device,
                            &mut encoder,
                            &surface.target_view,
                            &surface_view,
                        );
                        queue.submit(Some(encoder.finish()));
                        surface_texture.present();
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        .expect("Event loop failed");
}
