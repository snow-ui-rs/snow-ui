//! A simple button that updates the count by click.

use snow_ui::prelude::*;

#[derive(Message)]
struct IncreaseButtonClicked {}

widget! {
struct IncreaseButton {
    button: Button,
}
}

impl ClickHandler for IncreaseButton {
    async fn on_click(&mut self) {
        event_bus().send(IncreaseButtonClicked {});
    }
}

fn increase_button() -> Widget {
    widget![IncreaseButton {
        button: Button {
            text: "Increase Count",
        },
    }]
}

widget! {
struct SimpleText {
    count: u128,
}
}

impl MessageReceiver for SimpleText {
    async fn register(&mut self) {
        let mut rx = event_bus().subscribe::<IncreaseButtonClicked>();
        while rx.recv().await.is_ok() {
            self.count += 1;
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
                        children: widgets![increase_button(),],
                    },
                    Row {
                        children: widgets![SimpleText { count: 0 },],
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
