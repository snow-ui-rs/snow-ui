use std::sync::{Arc, Mutex};

use snow_ui::prelude::*;

fn sample_form(field: TextInput) -> Form {
    Form {
        children: vec![field.into()],
        ..Form::default()
    }
}

#[test]
fn form_json_contains_current_values_and_escapes_json_strings() {
    let field = TextInput {
        name: "message",
        label: "Message",
        ..TextInput::default()
    }
    .with_value("initial");
    field
        .value
        .set("quote: \" slash: \\ newline:\n雪".to_owned());
    let form = sample_form(field);

    let json = form.to_json().expect("form should serialize");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON expected");
    assert_eq!(
        parsed["fields"][0]["value"],
        "quote: \" slash: \\ newline:\n雪"
    );
    assert_eq!(parsed["fields"][0]["name"], "message");
    assert_eq!(
        form.values(),
        vec![(
            "message".to_owned(),
            "quote: \" slash: \\ newline:\n雪".to_owned()
        )]
    );
}

#[test]
fn submit_handler_observes_values_updated_after_form_conversion() {
    let received = Arc::new(Mutex::new(None));
    let received_by_handler = received.clone();
    let field = TextInput {
        name: "username",
        ..TextInput::default()
    };
    let form = Form {
        submit_handler: Arc::new(move |form: &Form| {
            let values = form.values();
            let received = received_by_handler.clone();
            async move {
                *received.lock().unwrap() = Some(values);
            }
        }),
        children: vec![field.clone().into()],
        ..Form::default()
    };

    let _object = form.clone().into_object();
    field.value.set("snow-user".to_owned());
    trigger_click_handler(form.submit_button.id);

    assert_eq!(
        *received.lock().unwrap(),
        Some(vec![("username".to_owned(), "snow-user".to_owned())])
    );
}

#[test]
fn reset_button_restores_initial_values() {
    let field = TextInput {
        name: "password",
        ..TextInput::default()
    }
    .with_value("starter");
    let form = Form {
        children: vec![field.clone().into()],
        ..Form::default()
    };
    let _object = form.clone().into_object();

    field.value.set("edited".to_owned());
    trigger_click_handler(form.reset_button.id);

    assert_eq!(field.value.get(), "starter");
}
