use vello::kurbo::Rect;
use vello::peniko::Color;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let _window = WindowBuilder::new()
        .with_title("Vello Research Example")
        .build(&event_loop)
        .expect("Failed to create window");

    let rect = Rect::new(0.0, 0.0, 128.0, 128.0);
    let color = Color::from_rgb8(0, 128, 255);
    println!(
        "Vello example started. Created a window and vello values: rect={:?}, color={:?}",
        rect, color
    );

    event_loop
        .run(move |event, event_loop_target| {
            event_loop_target.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        println!("Window close requested, exiting.");
                        std::process::exit(0);
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.logical_key == Key::Named(NamedKey::Escape) {
                            println!("Escape pressed, exiting.");
                            std::process::exit(0);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        .expect("Event loop failed");
}
