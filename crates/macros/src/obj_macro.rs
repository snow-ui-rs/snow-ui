//! Implementation of the `obj!` proc macro.

use quote::quote;

use crate::utils::add_defaults_to_expr;

/// Logic for `obj!(...)`.
///
/// Two modes:
/// - **Item mode**: `obj! { struct Foo { ... } }` — emits the struct and a stub `IntoObject` impl.
/// - **Expression mode**: `obj!(EXPR)` — adds `.. default()` to struct literals, wraps Form
///   submit handlers, then delegates to `__list_item!` for final processing.
pub(crate) fn expand(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    // Try to parse as a struct item first.
    if let Ok(item) = syn::parse2::<syn::ItemStruct>(input.clone()) {
        let name = &item.ident;
        return quote! {
            #item
            impl ::snow_ui::IntoObject for #name {
                fn into_object(self) -> ::snow_ui::Object {
                    unimplemented!("IntoObject not implemented for {}", stringify!(#name));
                }
            }
        };
    }

    // Otherwise parse as an expression.
    match syn::parse2::<syn::Expr>(input) {
        Ok(mut expr) => {
            if let syn::Expr::Struct(es) = &mut expr {
                // Recurse into fields to add defaults to nested struct literals.
                for field in es.fields.iter_mut() {
                    add_defaults_to_expr(&mut field.expr);
                }

                // If no `..rest` on the top-level struct, rebuild with defaults.
                if es.rest.is_none() {
                    let path = &es.path;
                    let fields: Vec<proc_macro2::TokenStream> =
                        es.fields.iter().map(|f| quote! { #f }).collect();
                    let rebuilt =
                        quote! { #path { #(#fields),* , .. ::snow_ui::prelude::default() } };
                    expr = syn::parse2(rebuilt).expect("failed to build nested defaulting struct");
                }
            }

            quote! {
                ::snow_ui_macros::__list_item!(#expr).into()
            }
        }
        Err(_) => {
            let msg = "obj! must be used either with a struct definition or an expression";
            syn::Error::new(proc_macro2::Span::call_site(), msg).to_compile_error()
        }
    }
}
