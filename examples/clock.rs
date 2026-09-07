//! A simple example showing object tree and built-in `TextClock` element.

use snow_ui::prelude::*;

pub fn world() -> World {
    World {
        root: obj!(Board {
            width: VIEWPORT_WIDTH,
            height: VIEWPORT_HEIGHT,
            h_align: HAlign::Center,
            v_align: VAlign::Middle,
            children: list![Card {
                children: list![
                    Row {
                        children: list![Text {
                            text: "Clock Example ⏰",
                        },],
                    },
                    Row {
                        children: list![TextClock { format: "%H:%M:%S" },],
                    },
                ],
            },],
        }),
        ..default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    snow_ui::launch(world);
}
