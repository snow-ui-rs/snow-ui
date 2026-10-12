use std::sync::atomic::{AtomicUsize, Ordering};

use snow_ui::prelude::*;

static FIRST_CLICKS: AtomicUsize = AtomicUsize::new(0);
static SECOND_CLICKS: AtomicUsize = AtomicUsize::new(0);

#[element]
struct FirstButton(Button);

impl ClickHandler for FirstButton {
    async fn on_click(&mut self) {
        FIRST_CLICKS.fetch_add(1, Ordering::SeqCst);
    }
}

#[element]
struct SecondButton(Button);

impl ClickHandler for SecondButton {
    async fn on_click(&mut self) {
        SECOND_CLICKS.fetch_add(1, Ordering::SeqCst);
    }
}

fn button_id(object: Object) -> WidgetId {
    match object {
        Object::Element(snow_ui::Element::Button(button)) => button.id,
        other => panic!("expected button object, got {other:?}"),
    }
}

#[test]
fn click_dispatch_invokes_only_the_handler_for_the_target_button() {
    FIRST_CLICKS.store(0, Ordering::SeqCst);
    SECOND_CLICKS.store(0, Ordering::SeqCst);

    let first_id = button_id(FirstButton(Button::new("First")).into_object());
    let second_id = button_id(SecondButton(Button::new("Second")).into_object());

    assert_ne!(first_id, second_id);

    trigger_click_handler(first_id);
    assert_eq!(FIRST_CLICKS.load(Ordering::SeqCst), 1);
    assert_eq!(SECOND_CLICKS.load(Ordering::SeqCst), 0);

    trigger_click_handler(second_id);
    assert_eq!(FIRST_CLICKS.load(Ordering::SeqCst), 1);
    assert_eq!(SECOND_CLICKS.load(Ordering::SeqCst), 1);

    trigger_click_handler(WidgetId::new());
    assert_eq!(FIRST_CLICKS.load(Ordering::SeqCst), 1);
    assert_eq!(SECOND_CLICKS.load(Ordering::SeqCst), 1);
}

#[test]
fn button_identity_is_unique_and_survives_cloning() {
    let original = Button::new("stable");
    let clone = original.clone();
    let another = Button::new("other");

    assert_eq!(original.id, clone.id);
    assert_ne!(original.id, another.id);
}
