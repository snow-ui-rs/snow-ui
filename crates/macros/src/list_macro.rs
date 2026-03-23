//! Implementation of `list!` and `__list_item!` proc macros.

use quote::quote;
use syn::parse::Parser;

use crate::utils::{is_form_path, process_struct_fields, rebuild_struct_with_defaults};

/// Logic for `__list_item!` — process a single expression, appending defaults to
/// struct literals that omit `..rest`.
pub(crate) fn list_item(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    match syn::parse2::<syn::Expr>(input) {
        Ok(mut e) => {
            if let syn::Expr::Struct(es) = &mut e
                && es.rest.is_none()
            {
                return rebuild_struct_with_defaults(es);
            }
            quote!(#e)
        }
        Err(e) => e.to_compile_error(),
    }
}

/// Logic for `list!` — parse comma-separated expressions and produce a `Vec`
/// with defaults appended to struct literals.
pub(crate) fn list(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::token::Comma>::parse_terminated;
    let exprs = match parser.parse2(input) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error(),
    };

    let mut out_exprs: Vec<proc_macro2::TokenStream> = Vec::new();
    for mut e in exprs.into_iter() {
        if let syn::Expr::Struct(es) = &mut e
            && es.rest.is_none()
        {
            let path = &es.path;
            let is_form = is_form_path(path);
            let fields_tokens = process_struct_fields(&es.fields, is_form);
            out_exprs
                .push(quote! { #path { #(#fields_tokens),* , .. ::snow_ui::prelude::default() } });
        } else if let syn::Expr::Struct(es) = &mut e {
            out_exprs.push(quote! { #es });
        } else {
            out_exprs.push(quote! { #e });
        }
    }

    quote! {
        vec![#(#out_exprs.into()),*]
    }
}
