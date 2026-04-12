use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let _window = WindowBuilder::new()
        .with_title("Winit Research Example")
        .build(&event_loop)
        .expect("Failed to create window");

    println!("Winit window created. Close the window to exit.");

    event_loop
        .run(move |event, event_loop_target| {
            event_loop_target.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => event_loop_target.exit(),
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.logical_key == Key::Named(NamedKey::Escape) {
                            event_loop_target.exit();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        .expect("Event loop failed");
}
