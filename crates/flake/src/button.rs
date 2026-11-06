use crate::{
    Bevel, Border, Color, LayoutRect, NativeEvent, PaintScene, PointerButton, Rect,
    RoundedRectNode, Shadow, TextNode, WidgetId,
};

const IDLE_COLOR: Color = Color::rgba(0.18, 0.65, 0.56, 1.0);
const ACTIVE_COLOR: Color = Color::rgba(0.93, 0.48, 0.20, 1.0);
const TEXT_COLOR: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);

/// A simple retained button that toggles its visual state when clicked.
#[derive(Debug, Clone)]
pub struct Button {
    id: WidgetId,
    bounds: Option<LayoutRect>,
    active: bool,
    pressed_pointer_id: Option<u64>,
}

impl Button {
    /// Create a button with a newly allocated identity shared by layout and paint nodes.
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            bounds: None,
            active: false,
            pressed_pointer_id: None,
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

    /// Return whether a pointer is currently pressing this button.
    pub const fn is_pressed(&self) -> bool {
        self.pressed_pointer_id.is_some()
    }

    /// Set the button's logical-pixel bounds, typically from the layout tree.
    pub fn set_bounds(&mut self, bounds: LayoutRect) {
        self.bounds = Some(bounds);
    }

    /// Forget the last layout bounds, such as when the surface is suspended.
    pub fn forget_bounds(&mut self) {
        self.bounds = None;
        self.pressed_pointer_id = None;
    }

    /// Handle a native event. Returns `true` when the button's visual state changes.
    pub fn handle_event(&mut self, event: &NativeEvent) -> bool {
        match event {
            NativeEvent::PointerButton {
                pointer_id,
                button: PointerButton::Primary,
                pressed: true,
                x,
                y,
                ..
            } => {
                if self.pressed_pointer_id.is_some() {
                    return false;
                }
                let Some(bounds) = self.bounds else {
                    return false;
                };
                let inside = *x >= f64::from(bounds.x)
                    && *x <= f64::from(bounds.x + bounds.width)
                    && *y >= f64::from(bounds.y)
                    && *y <= f64::from(bounds.y + bounds.height);
                if inside {
                    self.active = !self.active;
                    self.pressed_pointer_id = Some(*pointer_id);
                }
                inside
            }
            NativeEvent::PointerButton {
                pointer_id,
                button: PointerButton::Primary,
                pressed: false,
                ..
            } if self.pressed_pointer_id == Some(*pointer_id) => {
                self.pressed_pointer_id = None;
                true
            }
            NativeEvent::FocusChanged(false) => self.pressed_pointer_id.take().is_some(),
            _ => false,
        }
    }

    /// Add or update the button's rounded rectangle and label in the retained paint scene.
    pub fn paint(&self, scene: &mut PaintScene) {
        let Some(bounds) = self.bounds else {
            return;
        };
        let pressed = self.is_pressed();
        let y_offset = if pressed { 2.0 } else { 0.0 };
        let body_y = bounds.y + y_offset;

        scene.upsert_rounded_rect(RoundedRectNode {
            id: self.id,
            bounds: rect(bounds.x, body_y, bounds.width, bounds.height),
            corner_radius: 10.0,
            color: if self.active {
                ACTIVE_COLOR
            } else {
                IDLE_COLOR
            },
            border: Some(Border {
                width: 1.0,
                color: if pressed {
                    Color::rgba(0.06, 0.12, 0.14, 0.6)
                } else {
                    Color::rgba(0.94, 1.0, 0.98, 0.55)
                },
            }),
            shadow: Some(Shadow {
                offset_x: 0.0,
                offset_y: if pressed { 1.0 } else { 4.0 },
                blur_radius: if pressed { 2.0 } else { 5.0 },
                color: Color::rgba(0.0, 0.0, 0.0, 0.3),
            }),
            bevel: Some(Bevel {
                intensity: if pressed { 0.04 } else { 0.12 },
            }),
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
                body_y + (bounds.height - 22.0) / 2.0,
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

#[cfg(test)]
mod tests {
    use super::Button;
    use crate::{LayoutRect, NativeEvent, PaintScene, PointerButton, Rect, RefreshBatch};

    fn primary_button_event(pointer_id: u64, pressed: bool, x: f64, y: f64) -> NativeEvent {
        NativeEvent::PointerButton {
            pointer_id,
            button: PointerButton::Primary,
            pressed,
            x,
            y,
            target: None,
        }
    }

    #[test]
    fn buttons_generate_distinct_ids_and_clones_keep_the_id() {
        let first = Button::new();
        let second = Button::new();

        assert_ne!(first.id(), second.id());
        assert_eq!(first.id(), first.clone().id());
    }

    #[test]
    fn button_paints_as_a_rounded_rectangle() {
        let mut button = Button::new();
        button.set_bounds(LayoutRect {
            x: 10.0,
            y: 20.0,
            width: 220.0,
            height: 72.0,
        });
        let mut scene = PaintScene::default();
        button.paint(&mut scene);

        let frame = scene.take_frame(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 300.0,
                height: 200.0,
            },
            &RefreshBatch::default(),
        );

        assert!(frame.commands.is_empty());
        assert_eq!(frame.rounded_rects.len(), 1);
        assert_eq!(frame.rounded_rects[0].id, button.id());
        assert_eq!(frame.rounded_rects[0].corner_radius, 10.0);
        assert!(frame.rounded_rects[0].border.is_some());
        assert!(frame.rounded_rects[0].shadow.is_some());
        assert_eq!(frame.rounded_rects[0].shadow.unwrap().offset_y, 4.0);
        assert_eq!(frame.rounded_rects[0].bevel.unwrap().intensity, 0.12);
    }

    #[test]
    fn button_sinks_and_compresses_its_shadow_while_pressed() {
        let mut button = Button::new();
        button.set_bounds(LayoutRect {
            x: 10.0,
            y: 20.0,
            width: 220.0,
            height: 72.0,
        });
        let mut scene = PaintScene::default();
        button.paint(&mut scene);
        let _ = scene.take_frame(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 300.0,
                height: 200.0,
            },
            &RefreshBatch::default(),
        );

        assert!(button.handle_event(&primary_button_event(7, true, 30.0, 40.0)));
        assert!(button.is_pressed());
        assert!(button.is_active());
        button.paint(&mut scene);

        let frame = scene.take_frame(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 300.0,
                height: 200.0,
            },
            &RefreshBatch::default(),
        );
        let button_shape = &frame.rounded_rects[0];
        assert_eq!(button_shape.bounds.y, 22.0);
        let shadow = button_shape.shadow.unwrap();
        assert_eq!(shadow.offset_y, 1.0);
        assert_eq!(shadow.blur_radius, 2.0);
        assert_eq!(button_shape.bevel.unwrap().intensity, 0.04);
        assert_eq!(frame.text[0].bounds.y, 47.0);

        assert!(button.handle_event(&primary_button_event(7, false, 290.0, 190.0)));
        assert!(!button.is_pressed());
        button.paint(&mut scene);
        let frame = scene.take_frame(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 300.0,
                height: 200.0,
            },
            &RefreshBatch::default(),
        );
        assert_eq!(frame.rounded_rects[0].bounds.y, 20.0);
        assert_eq!(frame.rounded_rects[0].shadow.unwrap().offset_y, 4.0);
    }

    #[test]
    fn releasing_an_unrelated_pointer_does_not_unpress_the_button() {
        let mut button = Button::new();
        button.set_bounds(LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 40.0,
        });

        assert!(button.handle_event(&primary_button_event(1, true, 10.0, 10.0)));
        assert!(!button.handle_event(&primary_button_event(2, false, 10.0, 10.0)));
        assert!(button.is_pressed());
        assert!(button.handle_event(&NativeEvent::FocusChanged(false)));
        assert!(!button.is_pressed());
    }
}
