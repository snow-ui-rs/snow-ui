//! A simple little clock that updates the time every few milliseconds.

use snow_ui::prelude::*;

fn world() -> World {
    World {
        root: Board {
            width: VIEWPORT_WIDTH,
            height: VIEWPORT_HEIGHT,
            h_align: HAlign::Center,
            v_align: VAlign::Middle,
            children: vec![Card {
                children: vec![
                    Row {
                        children: vec![Text{
                            text: "Carpe diem 🎉",
                            ..default()
                        }.into()],
                        ..default()
                    },
                    Row {
                        children: vec![TextTimer{
                            format: "%H:%M:%S",
                            ..default()
                        }.into()],
                        ..default()
                    },
                ],
                ..default()
            }],
            ..default()
        }.into(),
    }
}

fn main() {
    snow_ui::launch(world);
}