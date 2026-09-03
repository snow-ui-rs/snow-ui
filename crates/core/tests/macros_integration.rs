use snow_ui::prelude::*;
use snow_ui::Element;

// Verify `#[element]` generates the hidden default factory and `impl Default` for non-generic structs
#[element]
struct FooElement {
    x: u32,
    y: Vec<u8>,
}

struct TestTick;

#[element]
struct VisibleElement {
    state: State<u32>,
    timer: IntervalTimer<TestTick>,
    text: Text,
}

#[test]
fn element_has_factory_and_default() {
    // The factory method should exist and set fields to their defaults
    let d = FooElement::__snow_ui_default();
    let d2 = FooElement::default();
    assert_eq!(d.x, 0);
    assert_eq!(d2.y.len(), 0);
}

#[test]
fn element_renders_only_visible_fields() {
    let state = State::new(7);
    let object = VisibleElement {
        state: state.clone(),
        timer: IntervalTimer::default(),
        text: Text::from_state(&state),
    }
    .into_object();

    match object {
        Object::Row(row) => {
            assert_eq!(row.children.len(), 1);
            assert!(matches!(row.children[0], Object::Element(Element::Text(_))));
        }
        other => panic!("expected visible fields to render as a row, got {other:?}"),
    }
}

// Verify `list!` proc-macro appends defaults to struct literals without `..` and mixes expressions
#[test]
fn list_macro_appends_defaults_and_mixes() {
    let v: Vec<Object> = list![Text { text: "hi" }, ::snow_ui::Object::from(1u128),];

    // Basic sanity: got two elements
    assert_eq!(v.len(), 2);
}

// Ensure our new `actions!` helper expands correctly and infers types.
#[test]
fn actions_macro_works() {
    let v: Vec<u8> = actions![1u8, 2u8,];
    assert_eq!(v, vec![1u8, 2u8]);
}
