use snow_ui::prelude::*;

#[test]
fn list_item_appends_default() {
    let _: Vec<::snow_ui::Object> = list![Text { text: "hi" }];
}

#[test]
fn textinput_has_label_and_form_is_available() {
    // verify `label` field on TextInput and the new Form element are usable in literals
    async fn handler(_: &Form) {}

    let _: Vec<Object> = list![Form {
        submit_handler: handler,
        submit_button: Button { text: "Login" },
        reset_button: Button { text: "Reset" },
        children: list![TextInput {
            label: "User:",
            name: "u",
            r#type: "text",
            max_len: 10
        },],
    }];
}

#[test]
fn switch_switch_to_changes_active_index() {
    let mut s = Switch {
        children: list![Text { text: "a" }, Text { text: "b" }],
        active: 0,
    };
    assert_eq!(s.active_index(), 0);
    s.switch_to(1);
    assert_eq!(s.active_index(), 1);
    s.switch_to(999); // out-of-range -> clamp
    assert_eq!(s.active_index(), 1);
}

#[test]
fn form_accepts_async_submit_handler() {
    // an async fn used directly as the submit handler should compile and be boxed by the macro
    async fn ahandler(_: &Form) {}

    let _: Vec<Object> = list![Form {
        submit_handler: ahandler,
        submit_button: Button { text: "OK" },
        reset_button: Button { text: "Reset" },
        children: vec![],
    }];
}
