use crate::{Color, LayoutRect, NativeEvent, PaintNode, PaintScene, PointerButton, Rect, TextNode, WidgetId};

const IDLE_COLOR: Color = Color::rgba(0.18, 0.65, 0.56, 1.0);
const ACTIVE_COLOR: Color = Color::rgba(0.93, 0.48, 0.20, 1.0);
const TEXT_COLOR: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);

/// A simple retained button that toggles its visual state when clicked.
#[derive(Debug, Clone)]
pub struct Button {
    id: WidgetId,
    bounds: Option<LayoutRect>,
    active: bool,
}

impl Button {
    /// Create a button with a stable identity shared by layout and paint nodes.
    pub const fn new(id: WidgetId) -> Self {
        Self {
            id,
            bounds: None,
            active: false,
        }
    }

    /// Return this button's stable identity.
    pub const fn id(&self) -> WidgetId {
        self.id
    }

    /// Return whether the button is currently toggled on.
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Set the button's logical-pixel bounds, typically from the layout tree.
    pub fn set_bounds(&mut self, bounds: LayoutRect) {
        self.bounds = Some(bounds);
    }

    /// Forget the last layout bounds, such as when the surface is suspended.
    pub fn forget_bounds(&mut self) {
        self.bounds = None;
    }

    /// Handle a native event. Returns `true` when a primary press hits the button.
    pub fn handle_event(&mut self, event: &NativeEvent) -> bool {
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

    /// Add or update the button's rectangle and label in the retained paint scene.
    pub fn paint(&self, scene: &mut PaintScene) {
        let Some(bounds) = self.bounds else {
            return;
        };

        scene.upsert(PaintNode {
            id: self.id,
            bounds: rect(bounds.x, bounds.y, bounds.width, bounds.height),
            color: if self.active {
                ACTIVE_COLOR
            } else {
                IDLE_COLOR
            },
            z_index: 0,
        });
        scene.upsert_text(TextNode {
            id: self.id,
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
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}