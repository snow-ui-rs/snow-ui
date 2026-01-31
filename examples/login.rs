//! A simple example showing object tree and built-in `TextClock` element.

use snow_ui::prelude::*;

#[element]
struct LoginBoard {
    board: Board,
}

fn login_board() -> Object {
    obj!(LoginBoard {
        board: Board {
            children: list![Card {
                children: list![
                    Row {
                        children: list![
                            Text {
                                text: "User name: ",
                            },
                            TextInput {
                                name: "username",
                                max_len: 20,
                            },
                        ],
                    },
                    Row {
                        children: list![
                            Text { text: "Password: " },
                            TextInput {
                                name: "password",
                                r#type: "password",
                                max_len: 20,
                            },
                        ],
                    },
                    Row {
                        children: list![Button { text: "Login" },],
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
