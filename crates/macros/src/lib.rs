use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(IntoObject, attributes(into_object))]
pub fn derive_into_object(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Parse optional helper attribute: `#[into_object(expr = "...")]` or `#[into_object(field = "field_name")]`
    let mut attr_expr: Option<syn::LitStr> = None;
    let mut attr_field: Option<syn::Ident> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("into_object") {
            // Fallback/simple parsing: convert tokens to string and look for `expr = "..."` and `field = "..."`.
            // This avoids depending on complicated syn::Meta APIs across versions.
            if let syn::Meta::List(list) = &attr.meta {
                let tokens_string = list.tokens.to_string();
                if let Some(idx) = tokens_string.find("expr") {
                    if let Some(q1) = tokens_string[idx..].find('"') {
                        let rest = &tokens_string[idx + q1 + 1..];
                        if let Some(q2) = rest.find('"') {
                            let val = &rest[..q2];
                            attr_expr = Some(syn::LitStr::new(val, proc_macro2::Span::call_site()));
                        }
                    }
                }
                if let Some(idx) = tokens_string.find("field") {
                    if let Some(q1) = tokens_string[idx..].find('"') {
                        let rest = &tokens_string[idx + q1 + 1..];
                        if let Some(q2) = rest.find('"') {
                            let val = &rest[..q2];
                            if let Ok(id) = syn::parse_str::<syn::Ident>(val) {
                                attr_field = Some(id);
                            }
                        }
                    }
                }
            }
        }
    }

    let expanded = match input.data {
        Data::Struct(ref s) => match &s.fields {
            // If the user provided an `expr` override, use it directly
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 && attr_expr.is_some() => {
                let expr = attr_expr.unwrap().value();
                let tokens: proc_macro2::TokenStream =
                    expr.parse().expect("failed to parse into_object expr");
                quote! {
                    impl ::snow_ui::IntoObject for #name {
                        fn into_object(self) -> ::snow_ui::Object {
                            #tokens
                        }
                    }
                }
            }
            // If user provided a named `field` override (handled in Fields::Named branch below), skip here
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field_ty = &fields.unnamed.iter().next().unwrap().ty;
                // Handle string-like fields specially to avoid requiring `IntoObject for String`.
                // - If it's a `&'static str`, use it directly.
                // - If it's a `&str` with a non-static lifetime, convert to owned and leak to `'static`.
                // - If it's a `String`, call `.into()` on the String value (let the field's conversion decide).
                // - Otherwise, fall back to calling `.into()` on the field value.
                if let syn::Type::Reference(r) = field_ty {
                    if let Some(lifetime) = &r.lifetime {
                        if lifetime.ident == "static" {
                            quote! {
                                impl ::snow_ui::IntoObject for #name {
                                    fn into_object(self) -> ::snow_ui::Object {
                                        ::snow_ui::Text { text: self.0 }.into()
                                    }
                                }
                            }
                        } else {
                            // non-'static &str -> to_owned() and leak
                            quote! {
                                impl ::snow_ui::IntoObject for #name {
                                    fn into_object(self) -> ::snow_ui::Object {
                                        let s: &'static str = Box::leak(self.0.to_owned().into_boxed_str());
                                        ::snow_ui::Text { text: s }.into()
                                    }
                                }
                            }
                        }
                    } else {
                        // reference without explicit lifetime - treat as non-static and leak
                        quote! {
                            impl ::snow_ui::IntoObject for #name {
                                fn into_object(self) -> ::snow_ui::Object {
                                    let s: &'static str = Box::leak(self.0.to_owned().into_boxed_str());
                                    ::snow_ui::Text { text: s }.into()
                                }
                            }
                        }
                    }
                } else if let syn::Type::Path(p) = field_ty {
                    // Check if the type is `String`.
                    if p.path.segments.last().unwrap().ident == "String" {
                        quote! {
                            impl ::snow_ui::IntoObject for #name {
                                fn into_object(self) -> ::snow_ui::Object {
                                    self.0.into()
                                }
                            }
                        }
                    } else {
                        // Fallback for other types: call `.into()` on the field value
                        quote! {
                            impl ::snow_ui::IntoObject for #name {
                                fn into_object(self) -> ::snow_ui::Object {
                                    self.0.into()
                                }
                            }
                        }
                    }
                } else {
                    quote! {
                        impl ::snow_ui::IntoObject for #name {
                            fn into_object(self) -> ::snow_ui::Object {
                                self.0.into()
                            }
                        }
                    }
                }
            }
            // If user provided a `field` override, honor it and generate conversion for that field
            Fields::Named(fields) if attr_field.is_some() => {
                let chosen = attr_field.as_ref().unwrap();
                // Find the actual field by name
                let actual_field = fields
                    .named
                    .iter()
                    .find(|f| f.ident.as_ref().map(|i| i == chosen).unwrap_or(false))
                    .expect("specified field not found");
                let field_ty = &actual_field.ty;

                if let syn::Type::Reference(r) = field_ty {
                    if let Some(lifetime) = &r.lifetime {
                        if lifetime.ident == "static" {
                            quote! {
                                impl ::snow_ui::IntoObject for #name {
                                    fn into_object(self) -> ::snow_ui::Object {
                                        ::snow_ui::Text { text: self.#chosen }.into()
                                    }
                                }
                            }
                        } else {
                            quote! {
                                impl ::snow_ui::IntoObject for #name {
                                    fn into_object(self) -> ::snow_ui::Object {
                                        let s: &'static str = Box::leak(self.#chosen.to_owned().into_boxed_str());
                                        ::snow_ui::Text { text: s }.into()
                                    }
                                }
                            }
                        }
                    } else {
                        quote! {
                            impl ::snow_ui::IntoObject for #name {
                                fn into_object(self) -> ::snow_ui::Object {
                                    let s: &'static str = Box::leak(self.#chosen.to_owned().into_boxed_str());
                                    ::snow_ui::Text { text: s }.into()
                                }
                            }
                        }
                    }
                } else if let syn::Type::Path(p) = field_ty {
                    if p.path.segments.last().unwrap().ident == "String" {
                        quote! {
                            impl ::snow_ui::IntoObject for #name {
                                fn into_object(self) -> ::snow_ui::Object {
                                    self.#chosen.into()
                                }
                            }
                        }
                    } else {
                        quote! {
                            impl ::snow_ui::IntoObject for #name {
                                fn into_object(self) -> ::snow_ui::Object {
                                    self.#chosen.into()
                                }
                            }
                        }
                    }
                } else {
                    quote! {
                        impl ::snow_ui::IntoObject for #name {
                            fn into_object(self) -> ::snow_ui::Object {
                                self.#chosen.into()
                            }
                        }
                    }
                }
            }
            _ => {
                quote! {
                    impl ::snow_ui::IntoObject for #name {
                        fn into_object(self) -> ::snow_ui::Object {
                            self.into()
                        }
                    }
                }
            }
        },
        _ => quote! {
            compile_error!("IntoObject can only be derived for structs");
        },
    };

    expanded.into()
}

#[proc_macro_derive(Message)]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = match input.data {
        Data::Struct(_) => {
            quote! {
                // Implement the marker `Message` trait from the core crate.
                impl ::snow_ui::Message for #name {}
            }
        }
        _ => quote! {
            compile_error!("Message can only be derived for structs");
        },
    };

    expanded.into()
}

/// Simple attribute macro form usable as `#[message] struct S { .. }`.
/// Emits the struct unchanged and implements the marker `::snow_ui::Message` for it.
#[proc_macro_attribute]
pub fn message(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse::<syn::ItemStruct>(item.clone()) {
        Ok(s) => {
            let name = &s.ident;
            let expanded = quote! {
                #s
                impl ::snow_ui::Message for #name {}
            };
            expanded.into()
        }
        Err(e) => e.to_compile_error().into(),
    }
}

/// A lightweight `obj!(...)` function-like proc-macro that supports two modes:
/// - Item mode: `obj! { struct Foo { ... } }` -> expands to the struct item and (optionally) generates an `IntoObject` impl
/// - Expression mode: `obj!(EXPR)` -> expands to `::snow_ui::__obj_expr!(EXPR)` so existing macro-rules handling (defaults, .into()) still applies.
#[proc_macro]
pub fn obj(input: TokenStream) -> TokenStream {
    // Try to parse as a struct item first
    if let Ok(item) = syn::parse::<syn::ItemStruct>(input.clone()) {
        let name = &item.ident;
        let expanded = quote! {
            #item
            impl ::snow_ui::IntoObject for #name {
                fn into_object(self) -> ::snow_ui::Object {
                    // Stubbed impl: no runtime logic yet.
                    unimplemented!("IntoObject not implemented for {}", stringify!(#name));
                }
            }
        };
        return expanded.into();
    }

    // Otherwise parse as an expression and forward to core's __obj_expr! macro to
    // preserve the earlier expression handling.
    match syn::parse::<syn::Expr>(input) {
        Ok(expr) => {
            let expanded = quote! {
                ::snow_ui::__obj_expr!(#expr)
            };
            expanded.into()
        }
        Err(_) => {
            // If we couldn't parse, emit a helpful compile error
            let msg = "obj! must be used either with a struct definition or an expression";
            syn::Error::new(proc_macro2::Span::call_site(), msg)
                .to_compile_error()
                .into()
        }
    }
}

/// Attribute form of `obj` usable as `#[element] struct Foo { ... }`.
/// This allows rustfmt to format the struct body normally (since it's a real item).
#[proc_macro_attribute]
pub fn element(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the item as a struct and generate a helpful `IntoObject` impl when possible.
    // If parsing fails, forward the syn error.
    match syn::parse::<syn::ItemStruct>(item.clone()) {
        Ok(s) => {
            let name = &s.ident;
            match &s.fields {
                syn::Fields::Unnamed(u) if u.unnamed.len() == 1 => {
                    // Tuple struct with a single field: forward to inner `.into()`.
                    // Special-case `Button` which converts to `Element` first.
                    let field_ty = &u.unnamed.iter().next().unwrap().ty;
                    let is_button = if let syn::Type::Path(p) = field_ty {
                        p.path.segments.last().unwrap().ident == "Button"
                    } else {
                        false
                    };

                    if is_button {
                        quote! {
                            #s
                            impl ::snow_ui::IntoObject for #name {
                                fn into_object(self) -> ::snow_ui::Object {
                                    let e: ::snow_ui::Element = self.0.into();
                                    e.into()
                                }
                            }
                        }
                        .into()
                    } else {
                        quote! {
                            #s
                            impl ::snow_ui::IntoObject for #name {
                                fn into_object(self) -> ::snow_ui::Object {
                                    self.0.into()
                                }
                            }
                        }
                        .into()
                    }
                }
                syn::Fields::Named(n) if n.named.len() == 1 => {
                    // Single named field: forward to that field's `.into()` conversion.
                    // Special-case `Button` to convert via `Element`.
                    let field = n.named.iter().next().unwrap();
                    let field_ident = field.ident.as_ref().unwrap();
                    let field_ty = &field.ty;
                    let is_button = if let syn::Type::Path(p) = field_ty {
                        p.path.segments.last().unwrap().ident == "Button"
                    } else {
                        false
                    };

                    if is_button {
                        quote! {
                            #s
                            impl ::snow_ui::IntoObject for #name {
                                fn into_object(self) -> ::snow_ui::Object {
                                    let e: ::snow_ui::Element = self.#field_ident.into();
                                    e.into()
                                }
                            }
                        }
                        .into()
                    } else {
                        quote! {
                            #s
                            impl ::snow_ui::IntoObject for #name {
                                fn into_object(self) -> ::snow_ui::Object {
                                    self.#field_ident.into()
                                }
                            }
                        }
                        .into()
                    }
                }
                _ => {
                    // Fallback: keep struct and emit an unimplemented stub so callers get a clear
                    // compile-time panic if they attempt to convert complex structs.
                    quote! {
                        #s
                        impl ::snow_ui::IntoObject for #name {
                            fn into_object(self) -> ::snow_ui::Object {
                                unimplemented!(concat!("IntoObject not implemented for ", stringify!(#name)));
                            }
                        }
                    }
                    .into()
                }
            }
        }
        Err(e) => e.to_compile_error().into(),
    }
}
