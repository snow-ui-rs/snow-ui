//! A simple clock example that uses the framework's built-in `TextClock` (framework-provided clock implementation).

use snow_ui::prelude::*;

fn world() -> World {
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

fn main() {
    snow_ui::launch(world);
}
