use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let _window = WindowBuilder::new()
        .with_title("Winit Close On Open")
        .build(&event_loop)
        .expect("Failed to create window");

    event_loop
        .run(move |event, event_loop_target| {
            event_loop_target.set_control_flow(ControlFlow::Wait);

            match event {
                Event::NewEvents(StartCause::Init) => {
                    event_loop_target.exit();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    event_loop_target.exit();
                }
                _ => {}
            }
        })
        .expect("Event loop failed");
}
