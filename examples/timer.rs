//! A simple little clock that updates the time every few milliseconds.

use snow_ui::prelude::*;
use tokio::time::{Duration, interval};

#[snow]
struct SimpleTextTimer {
    second: u128,
}

impl InnerTicker for SimpleTextTimer {
    async fn ticker(&mut self) {
        let mut iv = interval(Duration::from_secs(1));
        loop {
            iv.tick().await;
            self.second += 1;
        }
    }
}

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
                            text: "Timer Example ⏱️",
                        },],
                    },
                    Row {
                        children: widgets![SimpleTextTimer { second: 0 },],
                    },
                ],
            },],
        }],
        ..default()
    }
}

fn main() {
    snow_ui::launch(world);
}
