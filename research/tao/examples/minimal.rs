use tao::dpi::PhysicalSize;
use tao::event::{ElementState, Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::keyboard::KeyCode;
use tao::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new()
        .with_title("Tao Research Example")
        .with_inner_size(PhysicalSize::new(800, 600))
        .build(&event_loop)
        .expect("Failed to create window");

    println!("Tao window created. Close the window or press Escape to exit.");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } => {
                if event.state == ElementState::Pressed && event.physical_key == KeyCode::Escape {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}
