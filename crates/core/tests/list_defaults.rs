use snow_ui::prelude::*;

#[test]
fn list_item_appends_default() {
    let _: Vec<::snow_ui::Object> = list![Text { text: "hi" }];
}

#[test]
fn textinput_has_label_and_form_is_available() {
    // verify `label` field on TextInput and the new Form element are usable in literals
    fn handler(_: &Form) {}

    let _ : Vec<Object> = list![Form {
        submit_handler: handler,
        submit_button: Button { text: "Login" },
        reset_button: Button { text: "Reset" },
        children: list![
            TextInput { label: "User:", name: "u", r#type: "text", max_len: 10 },
        ],
    }];
}
