//! Shared helpers used by multiple macro implementations.

use quote::quote;

/// Given a struct-expression's fields iterator and whether the struct is a `Form`,
/// process fields and return token streams. For `Form` structs, bare function paths
/// assigned to `submit_handler` are wrapped with `Arc::new(...)` so they match the
/// expected `dyn SubmitHandler` type.
pub(crate) fn process_struct_fields(
    fields: &syn::punctuated::Punctuated<syn::FieldValue, syn::token::Comma>,
    is_form: bool,
) -> Vec<proc_macro2::TokenStream> {
    let mut tokens = Vec::new();
    for f in fields.iter() {
        if is_form
            && let syn::Member::Named(ident) = &f.member
            && ident == "submit_handler"
            && let syn::Expr::Path(_) = &f.expr
        {
            let expr = &f.expr;
            tokens.push(quote! {
                submit_handler: std::sync::Arc::new(
                    |form: &::snow_ui::Form| Box::pin({
                        let __owned = form.clone();
                        async move { (#expr)(&__owned).await }
                    })
                )
            });
            continue;
        }
        tokens.push(quote! { #f });
    }
    tokens
}

/// Returns `true` if `path`'s last segment is `"Form"`.
pub(crate) fn is_form_path(path: &syn::Path) -> bool {
    path.segments
        .last()
        .map(|s| s.ident == "Form")
        .unwrap_or(false)
}

/// Rebuild a struct-expression with `.. ::snow_ui::prelude::default()` appended,
/// processing Form submit_handler fields along the way.
pub(crate) fn rebuild_struct_with_defaults(es: &syn::ExprStruct) -> proc_macro2::TokenStream {
    let path = &es.path;
    let is_form = is_form_path(path);
    let fields_tokens = process_struct_fields(&es.fields, is_form);
    quote! { #path { #(#fields_tokens),* , .. ::snow_ui::prelude::default() } }
}

/// Recursively walk an expression tree and add `.. ::snow_ui::prelude::default()`
/// to struct literals that don't already have a `..rest`, and wrap `Form.submit_handler`
/// bare paths with `Arc::new(...)`.
pub(crate) fn add_defaults_to_expr(e: &mut syn::Expr) {
    match e {
        syn::Expr::Struct(es) => {
            // First recurse into fields so nested struct literals are also handled.
            for field in es.fields.iter_mut() {
                add_defaults_to_expr(&mut field.expr);
            }

            // Wrap Form submit_handler bare paths.
            if is_form_path(&es.path) {
                for field in es.fields.iter_mut() {
                    if let syn::Member::Named(ident) = &field.member
                        && ident == "submit_handler"
                        && let syn::Expr::Path(_) = &field.expr
                    {
                        let orig = &field.expr;
                        field.expr = syn::parse2(quote! {
                            std::sync::Arc::new(
                                |form: &::snow_ui::Form| Box::pin({
                                    let __owned = form.clone();
                                    async move { (#orig)(&__owned).await }
                                })
                            )
                        })
                        .expect("failed to wrap submit_handler");
                    }
                }
            }

            // Rebuild if no `..rest` is present.
            if es.rest.is_none() {
                let path = &es.path;
                let fields_tokens: Vec<proc_macro2::TokenStream> =
                    es.fields.iter().map(|f| quote! { #f }).collect();
                *e = syn::parse2(
                    quote! { #path { #(#fields_tokens),* , .. ::snow_ui::prelude::default() } },
                )
                .expect("failed to rebuild nested struct with defaults");
            }
        }
        syn::Expr::Array(arr) => {
            for elem in arr.elems.iter_mut() {
                add_defaults_to_expr(elem);
            }
        }
        syn::Expr::Call(call) => {
            for arg in call.args.iter_mut() {
                add_defaults_to_expr(arg);
            }
        }
        syn::Expr::Tuple(t) => {
            for elem in t.elems.iter_mut() {
                add_defaults_to_expr(elem);
            }
        }
        syn::Expr::Paren(p) => add_defaults_to_expr(&mut p.expr),
        syn::Expr::Reference(r) => add_defaults_to_expr(&mut r.expr),
        syn::Expr::Block(b) => {
            for stmt in b.block.stmts.iter_mut() {
                if let syn::Stmt::Expr(expr, _) = stmt {
                    add_defaults_to_expr(expr);
                }
            }
        }
        syn::Expr::Unary(u) => add_defaults_to_expr(&mut u.expr),
        syn::Expr::Binary(b) => {
            add_defaults_to_expr(&mut b.left);
            add_defaults_to_expr(&mut b.right);
        }
        _ => {}
    }
}
