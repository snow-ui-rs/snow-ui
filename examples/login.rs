//! A simple example showing object tree and built-in `TextClock` element.

use snow_ui::prelude::*;

#[element]
struct LoginBoard {
    board: Board,
}

async fn login(form: &Form) {
    // async handler (no-op for the example)
    let _ = form;
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

fn world() -> World {
    World {
        root: obj!(Switch {
            children: list![login_board(), main_board(),],
        }),
        ..default()
    }
}

fn main() {
    snow_ui::launch(world);
}
