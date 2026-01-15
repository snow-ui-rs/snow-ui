use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(IntoWidget, attributes(into_widget))]
pub fn derive_into_widget(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Parse optional helper attribute: `#[into_widget(expr = "...")]` or `#[into_widget(field = "field_name")]`
    let mut attr_expr: Option<syn::LitStr> = None;
    let mut attr_field: Option<syn::Ident> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("into_widget") {
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
                    expr.parse().expect("failed to parse into_widget expr");
                quote! {
                    impl ::snow_ui::IntoWidget for #name {
                        fn into_widget(self) -> ::snow_ui::Widget {
                            #tokens
                        }
                    }
                }
            }
            // If user provided a named `field` override (handled in Fields::Named branch below), skip here
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field_ty = &fields.unnamed.iter().next().unwrap().ty;
                // Handle string-like fields specially to avoid requiring `IntoWidget for String`.
                // - If it's a `&'static str`, use it directly.
                // - If it's a `&str` with a non-static lifetime, convert to owned and leak to `'static`.
                // - If it's a `String`, call `.into()` on the String value (let the field's conversion decide).
                // - Otherwise, fall back to calling `.into()` on the field value.
                if let syn::Type::Reference(r) = field_ty {
                    if let Some(lifetime) = &r.lifetime {
                        if lifetime.ident == "static" {
                            quote! {
                                impl ::snow_ui::IntoWidget for #name {
                                    fn into_widget(self) -> ::snow_ui::Widget {
                                        ::snow_ui::Text { text: self.0 }.into()
                                    }
                                }
                            }
                        } else {
                            // non-'static &str -> to_owned() and leak
                            quote! {
                                impl ::snow_ui::IntoWidget for #name {
                                    fn into_widget(self) -> ::snow_ui::Widget {
                                        let s: &'static str = Box::leak(self.0.to_owned().into_boxed_str());
                                        ::snow_ui::Text { text: s }.into()
                                    }
                                }
                            }
                        }
                    } else {
                        // reference without explicit lifetime - treat as non-static and leak
                        quote! {
                            impl ::snow_ui::IntoWidget for #name {
                                fn into_widget(self) -> ::snow_ui::Widget {
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
                            impl ::snow_ui::IntoWidget for #name {
                                fn into_widget(self) -> ::snow_ui::Widget {
                                    self.0.into()
                                }
                            }
                        }
                    } else {
                        // Fallback for other types: call `.into()` on the field value
                        quote! {
                            impl ::snow_ui::IntoWidget for #name {
                                fn into_widget(self) -> ::snow_ui::Widget {
                                    self.0.into()
                                }
                            }
                        }
                    }
                } else {
                    quote! {
                        impl ::snow_ui::IntoWidget for #name {
                            fn into_widget(self) -> ::snow_ui::Widget {
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
                                impl ::snow_ui::IntoWidget for #name {
                                    fn into_widget(self) -> ::snow_ui::Widget {
                                        ::snow_ui::Text { text: self.#chosen }.into()
                                    }
                                }
                            }
                        } else {
                            quote! {
                                impl ::snow_ui::IntoWidget for #name {
                                    fn into_widget(self) -> ::snow_ui::Widget {
                                        let s: &'static str = Box::leak(self.#chosen.to_owned().into_boxed_str());
                                        ::snow_ui::Text { text: s }.into()
                                    }
                                }
                            }
                        }
                    } else {
                        quote! {
                            impl ::snow_ui::IntoWidget for #name {
                                fn into_widget(self) -> ::snow_ui::Widget {
                                    let s: &'static str = Box::leak(self.#chosen.to_owned().into_boxed_str());
                                    ::snow_ui::Text { text: s }.into()
                                }
                            }
                        }
                    }
                } else if let syn::Type::Path(p) = field_ty {
                    if p.path.segments.last().unwrap().ident == "String" {
                        quote! {
                            impl ::snow_ui::IntoWidget for #name {
                                fn into_widget(self) -> ::snow_ui::Widget {
                                    self.#chosen.into()
                                }
                            }
                        }
                    } else {
                        quote! {
                            impl ::snow_ui::IntoWidget for #name {
                                fn into_widget(self) -> ::snow_ui::Widget {
                                    self.#chosen.into()
                                }
                            }
                        }
                    }
                } else {
                    quote! {
                        impl ::snow_ui::IntoWidget for #name {
                            fn into_widget(self) -> ::snow_ui::Widget {
                                self.#chosen.into()
                            }
                        }
                    }
                }
            }
            _ => {
                quote! {
                    impl ::snow_ui::IntoWidget for #name {
                        fn into_widget(self) -> ::snow_ui::Widget {
                            self.into()
                        }
                    }
                }
            }
        },
        _ => quote! {
            compile_error!("IntoWidget can only be derived for structs");
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

/// A lightweight `widget!(...)` function-like proc-macro that supports two modes:
/// - Item mode: `widget! { struct Foo { ... } }` -> expands to the struct item unchanged
/// - Expression mode: `widget!(EXPR)` -> expands to `::snow_ui::__widget_expr!(EXPR)` so
///   existing macro-rules handling (defaults, .into()) still applies.
#[proc_macro]
pub fn widget(input: TokenStream) -> TokenStream {
    // Try to parse as a struct item first
    if let Ok(item) = syn::parse::<syn::ItemStruct>(input.clone()) {
        let name = &item.ident;
        let expanded = quote! {
            #item
            impl ::snow_ui::IntoWidget for #name {
                fn into_widget(self) -> ::snow_ui::Widget {
                    // Stubbed impl: no runtime logic yet.
                    unimplemented!("IntoWidget not implemented for {}", stringify!(#name));
                }
            }
        };
        return expanded.into();
    }

    // Otherwise parse as an expression and forward to core's __widget_expr! macro to
    // preserve the earlier expression handling.
    match syn::parse::<syn::Expr>(input) {
        Ok(expr) => {
            let expanded = quote! {
                ::snow_ui::__widget_expr!(#expr)
            };
            expanded.into()
        }
        Err(_) => {
            // If we couldn't parse, emit a helpful compile error
            let msg = "widget! must be used either with a struct definition or an expression";
            syn::Error::new(proc_macro2::Span::call_site(), msg)
                .to_compile_error()
                .into()
        }
    }
}
