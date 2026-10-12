//! Implementation of `list!` and `__list_item!` proc macros.

use quote::quote;
use syn::parse::Parser;

use crate::utils::add_defaults_to_expr;

/// Logic for `__list_item!` — process a single expression, appending defaults to
/// struct literals that omit `..rest`.
pub(crate) fn list_item(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    match syn::parse2::<syn::Expr>(input) {
        Ok(mut expr) => {
            add_defaults_to_expr(&mut expr);
            quote!(#expr)
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
    for mut expr in exprs {
        // Struct defaults allocate stable WidgetIds, including for nested struct
        // literals handled by the shared recursive expression walker.
        add_defaults_to_expr(&mut expr);
        out_exprs.push(quote! { #expr });
    }

    quote! {
        vec![#(#out_exprs.into()),*]
    }
}
