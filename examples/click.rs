//! A simple button that updates the count by click.

use snow_ui::prelude::*;

#[derive(Message)]
struct IncreaseButtonClicked {}

#[derive(IntoWidget)]
struct IncreaseButton {
    button: Button,
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

#[derive(IntoWidget)]
struct SimpleText {
    count: u128,
}

impl MessageReceiver for SimpleText {
    async fn register(self) {
        let mut rx = event_bus().subscribe::<IncreaseButtonClicked>();
        let mut me = self;
        tokio::task::spawn_local(async move {
            while rx.recv().await.is_ok() {
                me.count += 1;
            }
        });
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
