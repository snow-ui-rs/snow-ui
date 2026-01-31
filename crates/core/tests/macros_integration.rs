use snow_ui::prelude::*;

// Verify `#[element]` generates the hidden default factory and `impl Default` for non-generic structs
#[element]
struct FooElement {
    x: u32,
    y: Vec<u8>,
}

#[test]
fn element_has_factory_and_default() {
    // The factory method should exist and set fields to their defaults
    let d = FooElement::__snow_ui_default();
    let d2 = FooElement::default();
    assert_eq!(d.x, 0);
    assert_eq!(d2.y.len(), 0);
}

// Verify `list!` proc-macro appends defaults to struct literals without `..` and mixes expressions
#[test]
fn list_macro_appends_defaults_and_mixes() {
    let v: Vec<Object> = list![Text { text: "hi" }, ::snow_ui::Object::from(1u128),];

    // Basic sanity: got two elements
    assert_eq!(v.len(), 2);
}
