// This crate's tests should not depend on the `snow-ui` core crate.
// The actual integration tests that exercise generated code live in
// `crates/core/tests` where `snow-ui` is available and re-exports the macros.

#[test]
fn smoke() {
    // simple sanity check so this test file compiles as part of macros crate tests
    assert!(true);
}
