#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use masonry::core::{ErasedAction, WidgetId};
use masonry::dpi::LogicalSize;
use masonry::widgets::ButtonPress;
use masonry_winit::app::{AppDriver, DriverCtx, NewWindow, WindowId};
use masonry_winit::winit::window::Window;
use snow_ui::backend::{SnowMessage, SnowNode, SnowWorld};
use snow_ui::masonry_backend::MasonryAdapter;

struct Driver {
    window_id: WindowId,
    adapter: MasonryAdapter,
}

impl AppDriver for Driver {
    fn on_action(
        &mut self,
        window_id: WindowId,
        _ctx: &mut DriverCtx<'_, '_>,
        _widget_id: WidgetId,
        action: ErasedAction,
    ) {
        debug_assert_eq!(window_id, self.window_id, "unknown window");

        if action.is::<ButtonPress>() {
            if let Some(message) = self.adapter.dispatch_action(&action) {
                self.adapter.handle_message(&message);
                let _rebuilt = self.adapter.render();
                match message {
                    SnowMessage::ButtonClicked { button_id, count } => {
                        println!("button {button_id} clicked, count={count}");
                    }
                }
            }
        } else {
            eprintln!("Unexpected action {action:?}");
        }
    }
}

fn main() {
    let mut adapter = MasonryAdapter::new();
    adapter.set_text(1, "Snow UI + Masonry");
    adapter.set_text(2, "Clicked 0 times");

    let world = SnowWorld {
        root: SnowNode::Column {
            children: vec![
                SnowNode::Text {
                    text: "Snow UI + Masonry".to_string(),
                    id: 1,
                },
                SnowNode::Button {
                    text: "Clicked 0 times".to_string(),
                    id: 2,
                },
            ],
        },
    };

    let main_widget = adapter.build_world(&world);

    let window_size = LogicalSize::new(400.0, 220.0);
    let window_attributes = Window::default_attributes()
        .with_title("Snow UI Masonry Button")
        .with_resizable(true)
        .with_min_inner_size(window_size);

    let driver = Driver {
        window_id: WindowId::next(),
        adapter,
    };

    masonry_winit::app::run(
        masonry_winit::app::EventLoop::with_user_event(),
        vec![NewWindow::new_with_id(
            driver.window_id,
            window_attributes,
            main_widget.erased(),
        )],
        driver,
        masonry::theme::default_property_set(),
    )
    .unwrap();
}
