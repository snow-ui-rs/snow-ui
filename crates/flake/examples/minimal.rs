use std::sync::Arc;

use snow_ui_flake::taffy::{AlignItems, Dimension, Display, JustifyContent, Size, Style};
use snow_ui_flake::{
    Color, LayoutNode, LayoutTree, NativeEvent, NativeEventReceiver, NativeEventSender,
    NativeRefreshHandle, NativeSurface, NativeUserEvent, PaintNode, PaintScene, PointerButton,
    RedrawProxy, TextNode, WinitEventAdapter, native_event_channel,
};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

const ROOT_ID: u64 = 1;
const BUTTON_ID: u64 = 2;
const BACKGROUND: Color = Color::rgba(0.08, 0.10, 0.14, 1.0);
const IDLE_COLOR: Color = Color::rgba(0.18, 0.65, 0.56, 1.0);
const ACTIVE_COLOR: Color = Color::rgba(0.93, 0.48, 0.20, 1.0);
const TEXT_COLOR: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);

struct MinimalApp {
    redraw_proxy: RedrawProxy,
    event_sender: NativeEventSender,
    event_receiver: NativeEventReceiver,
    window: Option<Arc<Window>>,
    surface: Option<NativeSurface>,
    event_adapter: Option<WinitEventAdapter>,
    refresh: Option<NativeRefreshHandle>,
    layout: LayoutTree,
    paint_scene: PaintScene,
    button_bounds: Option<snow_ui_flake::LayoutRect>,
    active: bool,
}

impl MinimalApp {
    fn new(event_loop: &EventLoop<NativeUserEvent>) -> Self {
        let root = LayoutNode::with_children(
            snow_ui_flake::WidgetId::from_raw(ROOT_ID),
            Style {
                display: Display::Flex,
                align_items: Some(AlignItems::Center),
                justify_content: Some(JustifyContent::Center),
                size: Size {
                    width: Dimension::percent(1.0),
                    height: Dimension::percent(1.0),
                },
                ..Style::default()
            },
            vec![LayoutNode::new(
                snow_ui_flake::WidgetId::from_raw(BUTTON_ID),
                Style {
                    size: Size {
                        width: Dimension::length(220.0),
                        height: Dimension::length(72.0),
                    },
                    ..Style::default()
                },
            )],
        );
        let (event_sender, event_receiver) = native_event_channel();
        Self {
            redraw_proxy: RedrawProxy::new(event_loop),
            event_sender,
            event_receiver,
            window: None,
            surface: None,
            event_adapter: None,
            refresh: None,
            layout: LayoutTree::new(root).expect("valid minimal layout tree"),
            paint_scene: PaintScene::default(),
            button_bounds: None,
            active: false,
        }
    }

    fn handle_native_event(&mut self, event: NativeEvent) {
        if let NativeEvent::PointerButton {
            button: PointerButton::Primary,
            pressed: true,
            x,
            y,
            ..
        } = event
            && self.button_bounds.is_some_and(|bounds| {
                x >= f64::from(bounds.x)
                    && x <= f64::from(bounds.x + bounds.width)
                    && y >= f64::from(bounds.y)
                    && y <= f64::from(bounds.y + bounds.height)
            })
        {
            self.active = !self.active;
            if let Some(refresh) = &self.refresh {
                let _ = refresh.invalidate_paint(snow_ui_flake::WidgetId::from_raw(BUTTON_ID));
            }
        }
    }

    fn drain_native_events(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_native_event(event);
        }
    }

    fn render(&mut self) {
        let (Some(surface), Some(refresh)) = (&mut self.surface, &self.refresh) else {
            return;
        };
        let Some([physical_width, physical_height]) = surface.physical_size() else {
            return;
        };
        let scale_factor = surface.scale_factor() as f32;
        let width = physical_width as f32 / scale_factor;
        let height = physical_height as f32 / scale_factor;
        let batch = refresh.take_pending();

        if (batch.full || !batch.layout_nodes.is_empty())
            && self.layout.compute_layout(width, height).is_err()
        {
            return;
        }
        let button_id = snow_ui_flake::WidgetId::from_raw(BUTTON_ID);
        let Ok(bounds) = self.layout.layout(button_id) else {
            return;
        };
        self.button_bounds = Some(bounds);
        self.paint_scene.upsert(PaintNode {
            id: button_id,
            bounds: snow_ui_flake::Rect {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
            },
            color: if self.active {
                ACTIVE_COLOR
            } else {
                IDLE_COLOR
            },
            z_index: 0,
        });
        self.paint_scene.upsert_text(TextNode {
            id: button_id,
            text: if self.active {
                "Clicked!"
            } else {
                "Click to change color"
            }
            .to_owned(),
            bounds: snow_ui_flake::Rect {
                x: bounds.x + 16.0,
                y: bounds.y + (bounds.height - 22.0) / 2.0,
                width: (bounds.width - 32.0).max(0.0),
                height: 22.0,
            },
            font_size: 16.0,
            line_height: 22.0,
            color: TEXT_COLOR,
            z_index: 1,
        });
        let frame = self.paint_scene.take_frame(
            snow_ui_flake::Rect {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
            &batch,
        );
        if let Err(error) = surface.render_frame(&frame, BACKGROUND) {
            eprintln!("native frame failed: {error:?}");
        }
    }
}

impl ApplicationHandler<NativeUserEvent> for MinimalApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Snow Flake Minimal"))
                .expect("failed to create window"),
        );
        let size = window.inner_size();
        let surface = pollster::block_on(NativeSurface::new(window.clone()))
            .expect("failed to initialize native surface");
        self.event_adapter = Some(WinitEventAdapter::new(
            window.scale_factor(),
            [size.width, size.height],
        ));
        self.refresh = Some(NativeRefreshHandle::new(
            self.redraw_proxy.clone(),
            window.id(),
        ));
        self.surface = Some(surface);
        self.window = Some(window);
        if let Some(refresh) = &self.refresh {
            let _ = refresh.invalidate_all();
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: NativeUserEvent) {
        match event {
            NativeUserEvent::Redraw(window_id) => {
                self.redraw_proxy.redraw_dispatched(window_id);
                if let Some(window) = &self.window
                    && window.id() == window_id
                {
                    window.request_redraw();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &self.window else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match &event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(surface) = &mut self.surface {
                    let _ = surface.resize(*size);
                }
                if let Some(refresh) = &self.refresh {
                    let _ = refresh.invalidate_all();
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let size = window.inner_size();
                if let Some(surface) = &mut self.surface {
                    let _ = surface.resize(size);
                }
                if let Some(refresh) = &self.refresh {
                    let _ = refresh.invalidate_all();
                }
            }
            WindowEvent::RedrawRequested => self.render(),
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && matches!(&event.logical_key, Key::Named(NamedKey::Escape)) =>
            {
                event_loop.exit();
            }
            _ => {}
        }

        if let Some(adapter) = &mut self.event_adapter
            && let Err(error) = adapter.dispatch(&event, &self.event_sender)
        {
            eprintln!("native event dispatch failed: {error}");
        }
        self.drain_native_events();
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.refresh = None;
        self.event_adapter = None;
        self.surface = None;
        self.window = None;
        self.button_bounds = None;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::<NativeUserEvent>::with_user_event().build()?;
    let mut app = MinimalApp::new(&event_loop);
    event_loop.run_app(&mut app)?;
    Ok(())
}
