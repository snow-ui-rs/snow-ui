//! Snow UI procedural macros.
//!
//! This crate is the proc-macro companion of `snow-ui`. All heavy lifting is
//! delegated to dedicated modules; this file is the thin entry-point required
//! by the compiler.

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod derive_into_object;
mod derive_message;
mod element_macro;
mod list_macro;
mod obj_macro;
mod utils;

// ── Derive macros ────────────────────────────────────────────────────────────

#[proc_macro_derive(IntoObject, attributes(into_object))]
pub fn derive_into_object(input: TokenStream) -> TokenStream {
    derive_into_object::derive(parse_macro_input!(input as syn::DeriveInput)).into()
}

#[proc_macro_derive(Message)]
pub fn derive_message(input: TokenStream) -> TokenStream {
    derive_message::derive(parse_macro_input!(input as syn::DeriveInput)).into()
}

// ── Attribute macros ─────────────────────────────────────────────────────────

/// `#[message] struct S { .. }` — emits the struct and implements `Message`.
#[proc_macro_attribute]
pub fn message(_attr: TokenStream, item: TokenStream) -> TokenStream {
    derive_message::attribute(item.into()).into()
}

/// `#[element]` / `#[element(message = [...])]` — emits the struct with
/// `Default`, `IntoObject`, and optional handler registration.
#[proc_macro_attribute]
pub fn element(attr: TokenStream, item: TokenStream) -> TokenStream {
    element_macro::expand(attr.into(), item.into()).into()
}

// ── Function-like macros ─────────────────────────────────────────────────────

/// Internal helper: process a single expression, appending defaults to struct
/// literals that omit `..rest`.
#[proc_macro]
pub fn __list_item(input: TokenStream) -> TokenStream {
    list_macro::list_item(input.into()).into()
}

/// `list![Foo { .. }, Bar { .. }]` — comma-separated struct literals with
/// automatic `.. default()` injection, producing a `Vec`.
#[proc_macro]
pub fn list(input: TokenStream) -> TokenStream {
    list_macro::list(input.into()).into()
}

/// `obj!(...)` — accepts either a struct definition or an expression and
/// produces an `Object`.
#[proc_macro]
pub fn obj(input: TokenStream) -> TokenStream {
    obj_macro::expand(input.into()).into()
}
