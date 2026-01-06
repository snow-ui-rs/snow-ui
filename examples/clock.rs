//! A simple little clock that updates the time every few milliseconds.

use snow_ui::prelude::*;

fn world() -> World {
    World {
        root: widget![Board {
            width: VIEWPORT_WIDTH,
            height: VIEWPORT_HEIGHT,
            h_align: HAlign::Center,
            v_align: VAlign::Middle,
            children: widgets![Card {
                children: widgets![
                    Row {
                        children: widgets![Text {
                            text: "Carpe diem 🎉",
                        },],
                    },
                    Row {
                        children: widgets![TextTimer { format: "%H:%M:%S" },],
                    },
                ],
            },],
        }],
    }
}

fn main() {
    snow_ui::launch(world);
}
