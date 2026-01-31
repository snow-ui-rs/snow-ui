// Keep this file a compile-only smoke test; integration tests that rely on the
// `snow-ui` crate live in `crates/core/tests` so `crates/macros` does not depend on
// `crates/core` in dev-dependencies.

#[test]
fn smoke() {
    assert!(true);
}
