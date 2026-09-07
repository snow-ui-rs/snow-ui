//! A minimal example demonstrating message transfer between components.

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
    count: State<u128>,
    text: Text,
}

register_handler!(
    impl MessageHandler<IncreaseButtonClicked> for SimpleText {
        async fn handle(&mut self, _: &IncreaseButtonClicked, _: &mut MessageContext) {
            self.count.update(|c| *c += 1);
        }
    }
);

fn simple_text() -> Object {
    let count = State::new(0);
    obj!(SimpleText {
        count: count.clone(),
        text: Text::from_state(&count),
    })
}

pub fn world() -> World {
    World {
        root: obj!(Board {
            children: list![Card {
                children: list![
                    Row {
                        children: list![increase_button(),],
                    },
                    Row {
                        children: list![simple_text(),],
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
