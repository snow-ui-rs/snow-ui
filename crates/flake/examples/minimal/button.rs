use snow_ui_flake::{
    Color, LayoutRect, NativeEvent, PaintNode, PaintScene, PointerButton, TextNode,
};

const BUTTON_ID: u64 = 2;
const IDLE_COLOR: Color = Color::rgba(0.18, 0.65, 0.56, 1.0);
const ACTIVE_COLOR: Color = Color::rgba(0.93, 0.48, 0.20, 1.0);
const TEXT_COLOR: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);

pub(super) struct ExampleButton {
    bounds: Option<LayoutRect>,
    active: bool,
}

impl ExampleButton {
    pub(super) fn new() -> Self {
        Self {
            bounds: None,
            active: false,
        }
    }

    pub(super) fn id(&self) -> snow_ui_flake::WidgetId {
        snow_ui_flake::WidgetId::from_raw(BUTTON_ID)
    }

    pub(super) fn handle_event(&mut self, event: &NativeEvent) -> bool {
        let NativeEvent::PointerButton {
            button: PointerButton::Primary,
            pressed: true,
            x,
            y,
            ..
        } = event
        else {
            return false;
        };

        let Some(bounds) = self.bounds else {
            return false;
        };
        let inside = *x >= f64::from(bounds.x)
            && *x <= f64::from(bounds.x + bounds.width)
            && *y >= f64::from(bounds.y)
            && *y <= f64::from(bounds.y + bounds.height);
        if inside {
            self.active = !self.active;
        }
        inside
    }

    pub(super) fn paint(&self, bounds: LayoutRect, scene: &mut PaintScene) {
        let id = self.id();
        scene.upsert(PaintNode {
            id,
            bounds: rect(bounds.x, bounds.y, bounds.width, bounds.height),
            color: if self.active {
                ACTIVE_COLOR
            } else {
                IDLE_COLOR
            },
            z_index: 0,
        });
        scene.upsert_text(TextNode {
            id,
            text: if self.active {
                "Clicked!"
            } else {
                "Click to change color"
            }
            .to_owned(),
            bounds: rect(
                bounds.x + 16.0,
                bounds.y + (bounds.height - 22.0) / 2.0,
                (bounds.width - 32.0).max(0.0),
                22.0,
            ),
            font_size: 16.0,
            line_height: 22.0,
            color: TEXT_COLOR,
            z_index: 1,
        });
    }

    pub(super) fn forget_bounds(&mut self) {
        self.bounds = None;
    }

    pub(super) fn set_bounds(&mut self, bounds: LayoutRect) {
        self.bounds = Some(bounds);
    }
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> snow_ui_flake::Rect {
    snow_ui_flake::Rect {
        x,
        y,
        width,
        height,
    }
}
