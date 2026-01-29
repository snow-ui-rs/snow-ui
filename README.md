# Snow UI: Pure Rust UI Framework

## Philosophy

The UI is a projection of a virtual world onto the screen.


## Example: A lovely girl

A minimal component example demonstrating the `#[element]` attribute  and constructing a component with several fields.

```rust
use snow_ui::prelude::*;

#[element]
struct LovelyGirl {
    girl: Girl,
}

fn lovely_girl() -> Object {
    obj!(LovelyGirl {
        girl: Girl {
            hair_color: HairColor::Black,
            skin_color: SkinColor::Yellow,
            body_type: BodyType::Slim,
            appearance: Appearance::Beautiful,
            every_morning: vec![GirlActions::SayHi, GirlActions::PrepareBreakfast,],
        },
    })
}

fn world() -> World {
    World {
        root: lovely_girl(),
        ..default()
    }
}

fn main() {
    snow_ui::launch(world);
}
```

---

## Example: Text Clock

A simple board example: a `Board` containing a `Card` with two rows (`Text` and `TextClock`).

```rust
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
```

---

## Example: Timer (user ticker)

Implements its own ticker via `tokio::time::interval`; seconds are stored in `State<u128>`.

```rust
use snow_ui::prelude::*;
use tokio::time::{Duration, interval};

#[element]
struct SimpleTextTimer {
    second: State<u128>,
}

impl InnerTicker for SimpleTextTimer {
    async fn ticker(&mut self) {
        let mut iv = interval(Duration::from_secs(1));
        loop {
            iv.tick().await;
            self.second.update(|s| *s += 1);
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
                        children: list![Text {
                            text: "Timer Example ⏱️",
                        },],
                    },
                    Row {
                        children: list![SimpleTextTimer { second: State::new(0) },],
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
```

---

## Example: Button Click (event)

Button sends an event to the global event bus; handler registered with `register_handler!`.

```rust
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
}

// Use register_handler! to automatically register the handler via inventory
register_handler!(
    impl MessageHandler<IncreaseButtonClicked> for SimpleText {
        async fn handle(&mut self, _: &IncreaseButtonClicked, _: &mut MessageContext) {
            // Update the state via `update` helper
            self.count.update(|c| *c += 1);
        }
    }
);

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
                        children: list![SimpleText { count: State::new(0) },],
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
```

