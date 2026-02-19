# Snow UI: Pure Rust UI Framework

## Philosophy

The UI is a projection of a virtual world onto the screen.


## Example: A lovely girl

A minimal example demonstrating a virtual world and a typical element with properties and actions.

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

A simple example showing object tree and built-in `TextClock` element.

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

## Example: Timer (custom implementation)

A simple timer example that implements its own ticker.

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
                        children: list![SimpleTextTimer {
                            second: State::new(0)
                        },],
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

A minimal example demonstrating message transfer between components.

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

register_handler!(
    impl MessageHandler<IncreaseButtonClicked> for SimpleText {
        async fn handle(&mut self, _: &IncreaseButtonClicked, _: &mut MessageContext) {
            self.count.update(|c| *c += 1);
        }
    }
);

fn simple_text() -> Object {
    obj!(SimpleText {
        count: State::new(0)
    })
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
                        children: list![simple_text(),],
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

## Example: Login (forms, async handlers, ServerApi)

A compact login example demonstrating `Form`, `TextInput`, `ServerApi` and `Switch` + message handling.

```rust
use snow_ui::prelude::*;

#[message]
struct LoginSuccess {}

#[element]
struct LoginBoard {
    board: Board,
}

async fn login(form: &Form) -> anyhow::Result<()> {
    let json = form.to_json()?;
    let server_api = ServerApi::new("https://httpbin.org/post");
    let resp = server_api.post_json(json).await?;
    println!("Server response: {}", resp);
    event_bus().send(LoginSuccess {});
    Ok(())
}

fn login_board() -> Object {
    obj!(LoginBoard {
        board: Board {
            children: list![Form {
                submit_handler: login,
                submit_button: Button { text: "Login" },
                reset_button: Button { text: "Reset" },
                children: list![
                    Row {
                        children: list![TextInput {
                            label: "User name: ",
                            name: "username",
                            max_len: 20,
                        },],
                    },
                    Row {
                        children: list![TextInput {
                            label: "Password: ",
                            name: "password",
                            r#type: "password",
                            max_len: 20,
                        },],
                    },
                ],
            },],
        }
    })
}

#[element]
struct MainBoard {
    board: Board,
}

fn main_board() -> Object {
    obj!(MainBoard {
        board: Board {
            children: list![Card {
                children: list![Text {
                    text: "Welcome to the main board!",
                },],
            },],
        }
    })
}

#[element]
struct MySwitch {
    switch: Switch,
}

register_handler!(
    impl MessageHandler<LoginSuccess> for MySwitch {
        async fn handle(&mut self, _: &LoginSuccess, _: &mut MessageContext) {
            self.switch.switch_to(1);
        }
    }
);

fn my_switch() -> Object {
    obj!(MySwitch {
        switch: Switch {
            children: list![login_board(), main_board(),],
        }
    })
}

fn world() -> World {
    World {
        root: my_switch(),
        ..default()
    }
}

fn main() {
    snow_ui::launch(world);
}
```

