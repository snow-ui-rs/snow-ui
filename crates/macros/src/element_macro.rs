//! Implementation of the `#[element]` attribute macro.

use quote::quote;

/// Logic for `#[element]` / `#[element(message = [...])]`.
///
/// Emits the struct unchanged, generates `Default` (if missing), a hidden factory,
/// and an `IntoObject` impl that optionally registers message handlers.
pub(crate) fn expand(
    attr: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    // ── Parse message/register attribute ─────────────────────────────────
    let message_paths = parse_message_paths(attr);

    // ── Parse the struct item ────────────────────────────────────────────
    let s = match syn::parse2::<syn::ItemStruct>(item) {
        Ok(s) => s,
        Err(e) => return e.to_compile_error(),
    };

    let name = &s.ident;
    let has_default = has_derive_default(&s);
    let struct_item = quote! { #s };

    // ── Default impl / factory ───────────────────────────────────────────
    let default_impl = gen_default_impl(&s, name, has_default);

    // ── IntoObject impl ──────────────────────────────────────────────────
    gen_into_object(&s, name, &struct_item, &default_impl, &message_paths)
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Parse `message = [A, B]`, `message = "A, B"`, `register = ...` from the
/// attribute tokens and return the collected paths.
fn parse_message_paths(attr: proc_macro2::TokenStream) -> Vec<syn::Path> {
    let mut paths = Vec::new();
    if attr.is_empty() {
        return paths;
    }

    let s = attr.to_string();
    for key in ["message", "register"] {
        let mut start = 0usize;
        while let Some(pos) = s[start..].find(key) {
            let idx = start + pos;
            if let Some(eq_pos) = s[idx..].find('=') {
                let after_eq = idx + eq_pos + 1;
                let rest = s[after_eq..].trim_start();

                if rest.starts_with('[') {
                    if let Some(end) = rest.find(']') {
                        collect_paths(&rest[1..end], &mut paths);
                    }
                } else if rest.starts_with('"') {
                    if let Some(end) = rest[1..].find('"') {
                        collect_paths(&rest[1..1 + end], &mut paths);
                    }
                } else {
                    let token: String = rest.chars().take_while(|&c| c != ',' && c != ')' && c != ']').collect();
                    collect_paths(token.trim(), &mut paths);
                }
            }
            start = idx + key.len();
        }
    }
    paths
}

fn collect_paths(input: &str, out: &mut Vec<syn::Path>) {
    for part in input.split(',') {
        let p = part.trim();
        if !p.is_empty() {
            if let Ok(path) = syn::parse_str::<syn::Path>(p) {
                out.push(path);
            }
        }
    }
}

fn has_derive_default(s: &syn::ItemStruct) -> bool {
    for attr in &s.attrs {
        if attr.path().is_ident("derive") {
            if let syn::Meta::List(list) = &attr.meta {
                if list.tokens.to_string().contains("Default") {
                    return true;
                }
            }
        }
    }
    false
}

/// Generate `__snow_ui_default()` factory and optionally `impl Default`.
fn gen_default_impl(
    s: &syn::ItemStruct,
    name: &syn::Ident,
    has_default: bool,
) -> proc_macro2::TokenStream {
    // Only generate for non-generic structs.
    if !s.generics.params.is_empty() {
        return quote! {};
    }

    let factory = match &s.fields {
        syn::Fields::Named(n) if !n.named.is_empty() => {
            let assigns = n.named.iter().map(|f| {
                let ident = &f.ident;
                quote! { #ident: ::std::default::Default::default() }
            });
            quote! {
                impl #name {
                    #[doc(hidden)]
                    fn __snow_ui_default() -> Self {
                        #name { #(#assigns),* }
                    }
                }
            }
        }
        syn::Fields::Unnamed(u) if !u.unnamed.is_empty() => {
            let defaults =
                (0..u.unnamed.len()).map(|_| quote! { ::std::default::Default::default() });
            quote! {
                impl #name {
                    #[doc(hidden)]
                    fn __snow_ui_default() -> Self {
                        #name( #(#defaults),* )
                    }
                }
            }
        }
        syn::Fields::Unit => {
            quote! {
                impl #name {
                    #[doc(hidden)]
                    fn __snow_ui_default() -> Self { #name }
                }
            }
        }
        _ => quote! {},
    };

    if has_default {
        factory
    } else {
        quote! {
            #factory
            impl ::std::default::Default for #name {
                fn default() -> Self { #name::__snow_ui_default() }
            }
        }
    }
}

/// Generate the `IntoObject` impl, dispatching on field shape and whether there
/// are explicit message registrations vs inventory-based auto-registration.
fn gen_into_object(
    s: &syn::ItemStruct,
    name: &syn::Ident,
    struct_item: &proc_macro2::TokenStream,
    default_impl: &proc_macro2::TokenStream,
    message_paths: &[syn::Path],
) -> proc_macro2::TokenStream {
    match &s.fields {
        syn::Fields::Unnamed(u) if u.unnamed.len() == 1 => {
            let field_ty = &u.unnamed.iter().next().unwrap().ty;
            let is_button = is_button_ty(field_ty);
            gen_single_field_into_object(
                name,
                struct_item,
                default_impl,
                message_paths,
                is_button,
                None, // unnamed: accessor is .0
            )
        }
        syn::Fields::Named(n) if n.named.len() == 1 => {
            let field = n.named.iter().next().unwrap();
            let field_ident = field.ident.as_ref().unwrap();
            let is_button = is_button_ty(&field.ty);
            gen_single_field_into_object(
                name,
                struct_item,
                default_impl,
                message_paths,
                is_button,
                Some(field_ident),
            )
        }
        _ => {
            // Multi-field or unit struct — inventory-based fallback.
            quote! {
                #struct_item
                #default_impl
                impl ::snow_ui::IntoObject for #name {
                    fn into_object(self) -> ::snow_ui::Object {
                        if ::snow_ui::has_registered_handlers::<#name>() {
                            let rc = ::std::rc::Rc::new(::std::cell::RefCell::new(self));
                            ::snow_ui::register_handlers_for_instance(&rc);
                        }
                        ::snow_ui::Text { text: "" }.into()
                    }
                }
            }
        }
    }
}

fn is_button_ty(ty: &syn::Type) -> bool {
    if let syn::Type::Path(p) = ty {
        p.path.segments.last().unwrap().ident == "Button"
    } else {
        false
    }
}

/// Generate `IntoObject` for a single-field struct (named or unnamed).
/// `field_ident` is `None` for tuple structs (use `.0`) and `Some(ident)` for named.
fn gen_single_field_into_object(
    name: &syn::Ident,
    struct_item: &proc_macro2::TokenStream,
    default_impl: &proc_macro2::TokenStream,
    message_paths: &[syn::Path],
    is_button: bool,
    field_ident: Option<&syn::Ident>,
) -> proc_macro2::TokenStream {
    // Build the accessor expression for the inner field.
    let accessor = match field_ident {
        Some(id) => quote! { #id },
        None => quote! { 0 },
    };

    // Build the conversion body.  For buttons the value goes through `Element`; for
    // everything else `.into()` suffices.
    let (value_from_ref, value_from_self) = if is_button {
        (
            quote! { let e: ::snow_ui::Element = rc.borrow().#accessor.clone().into(); e.into() },
            quote! { let e: ::snow_ui::Element = self.#accessor.into(); e.into() },
        )
    } else {
        (
            quote! { rc.borrow().#accessor.clone().into() },
            quote! { self.#accessor.into() },
        )
    };

    if message_paths.is_empty() {
        // Inventory-based auto-registration.
        quote! {
            #struct_item
            #default_impl
            impl ::snow_ui::IntoObject for #name {
                fn into_object(self) -> ::snow_ui::Object {
                    if ::snow_ui::has_registered_handlers::<#name>() {
                        let rc = ::std::rc::Rc::new(::std::cell::RefCell::new(self));
                        ::snow_ui::register_handlers_for_instance(&rc);
                        #value_from_ref
                    } else {
                        #value_from_self
                    }
                }
            }
        }
    } else {
        let regs = message_paths.iter();
        quote! {
            #struct_item
            #default_impl
            impl ::snow_ui::IntoObject for #name {
                fn into_object(self) -> ::snow_ui::Object {
                    let rc = ::std::rc::Rc::new(::std::cell::RefCell::new(self));
                    #(
                        ::snow_ui::event_bus().register_handler::<#name, #regs>(rc.clone());
                    )*
                    #value_from_ref
                }
            }
        }
    }
}
