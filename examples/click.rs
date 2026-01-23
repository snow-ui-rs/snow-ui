//! A simple button that updates the count by click.

use snow_ui::prelude::*;

#[message]
struct IncreaseButtonClicked {}

#[element]
struct IncreaseButton {
    button: Button,
}

impl ClickHandler for IncreaseButton {
    async fn on_click(&mut self) {
        event_bus().send(IncreaseButtonClicked {});
    }
}

fn increase_button() -> Object {
    obj!(IncreaseButton {
        button: Button {
            text: "Increase Count",
        },
    })
}

#[element]
struct SimpleText {
    count: u128,
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
        root: obj!(Board {
            width: VIEWPORT_WIDTH,
            height: VIEWPORT_HEIGHT,
            h_align: HAlign::Center,
            v_align: VAlign::Middle,
            children: list![Card {
                children: list![
                    Row {
                        children: list![increase_button(),],
                    },
                    Row {
                        children: list![SimpleText { count: 0 },],
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
