//! A simple example showing object tree and built-in `TextClock` element.

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
                submit_button: Button {
                    text: "Login",
                },
                reset_button: Button {
                    text: "Reset",
                },
                children: list![
                    Row {
                        children: list![
                            TextInput {
                                label: "User name: ",
                                name: "username",
                                max_len: 20,
                            },
                        ],
                    },
                    Row {
                        children: list![
                            TextInput {
                                label: "Password: ",
                                name: "password",
                                r#type: "password",
                                max_len: 20,
                            },
                        ],
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
