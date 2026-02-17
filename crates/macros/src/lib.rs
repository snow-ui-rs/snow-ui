use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
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
                                        ::snow_ui::Text { text: self.0, .. ::snow_ui::default() }.into()
                                    }
                                }
                            }
                        } else {
                            // non-'static &str -> to_owned() and leak
                            quote! {
                                impl ::snow_ui::IntoObject for #name {
                                    fn into_object(self) -> ::snow_ui::Object {
                                        let s: &'static str = Box::leak(self.0.to_owned().into_boxed_str());
                                        ::snow_ui::Text { text: s, .. ::snow_ui::default() }.into()
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
                                        ::snow_ui::Text { text: self.#chosen, .. ::snow_ui::default() }.into()
                                    }
                                }
                            }
                        } else {
                            quote! {
                                impl ::snow_ui::IntoObject for #name {
                                    fn into_object(self) -> ::snow_ui::Object {
                                        let s: &'static str = Box::leak(self.#chosen.to_owned().into_boxed_str());
                                        ::snow_ui::Text { text: s, .. ::snow_ui::default() }.into()
                                    }
                                }
                            }
                        }
                    } else {
                        quote! {
                            impl ::snow_ui::IntoObject for #name {
                                fn into_object(self) -> ::snow_ui::Object {
                                    let s: &'static str = Box::leak(self.#chosen.to_owned().into_boxed_str());
                                    ::snow_ui::Text { text: s, .. ::snow_ui::default() }.into()
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

/// Proc-macro version of `list!` that parses comma-separated expressions and
/// automatically appends `.. default()` to struct literals that omit `..rest`.
#[proc_macro]
pub fn __list_item(input: TokenStream) -> TokenStream {
    // Accept an expression; if it's a struct literal without `..`, append defaults.
    match syn::parse::<syn::Expr>(input) {
        Ok(mut e) => {
            if let syn::Expr::Struct(es) = &mut e {
                if es.rest.is_none() {
                    // Rebuild with explicit comma before `..` to avoid range parsing
                    let path = &es.path;

                    // Special-case `Form { submit_handler: foo, ... }` so bare function
                    // paths (e.g. `login`) are wrapped into `Box::new(...)` to coerce
                    // into the object-safe handler type in core.
                    let is_form = path.segments.last().map(|s| s.ident == "Form").unwrap_or(false);
                    let mut fields_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
                    for f in es.fields.iter() {
                        if is_form {
                            if let syn::Member::Named(ident) = &f.member {
                                if ident == "submit_handler" {
                                    match &f.expr {
                                        syn::Expr::Path(_) => {
                                            let expr = &f.expr;
                                            fields_tokens.push(quote! { submit_handler: std::sync::Arc::new(|form: &::snow_ui::Form| Box::pin({ let __owned = form.clone(); async move { (#expr)(&__owned).await } })) });
                                            continue;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        fields_tokens.push(quote! { #f });
                    }

                    let rebuilt = quote! { #path { #(#fields_tokens),* , .. ::snow_ui::prelude::default() } };
                    return syn::parse2(rebuilt)
                        .unwrap_or_else(|e| e.to_compile_error())
                        .into();
                }
            }
            quote!(#e).into()
        }
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn list(input: TokenStream) -> TokenStream {
    // Parse a comma-separated list of expressions
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::token::Comma>::parse_terminated;
    let exprs = match parser.parse2(input.into()) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut out_exprs: Vec<proc_macro2::TokenStream> = Vec::new();
    for mut e in exprs.into_iter() {
        if let syn::Expr::Struct(es) = &mut e {
            if es.rest.is_none() {
                // Build a struct literal token stream that appends `.. ::snow_ui::prelude::default()`
                // and ensures there's a comma before the `..` so it doesn't parse as a range.
                let path = &es.path;
                let is_form = path.segments.last().map(|s| s.ident == "Form").unwrap_or(false);
                let mut fields_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
                for f in es.fields.iter() {
                    if is_form {
                        if let syn::Member::Named(ident) = &f.member {
                            if ident == "submit_handler" {
                                match &f.expr {
                                    syn::Expr::Path(_) => {
                                        let expr = &f.expr;
                                        fields_tokens.push(quote! { submit_handler: std::sync::Arc::new(|form: &::snow_ui::Form| Box::pin({ let __owned = form.clone(); async move { (#expr)(&__owned).await } })) });
                                        continue;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    fields_tokens.push(quote! { #f });
                }
                out_exprs.push(quote! { #path { #(#fields_tokens),* , .. ::snow_ui::prelude::default() } });
            } else {
                out_exprs.push(quote! { #es });
            }
        } else {
            out_exprs.push(quote! { #e });
        }
    }

    let expanded = quote! {
        vec![#(#out_exprs.into()),*]
    };

    expanded.into()
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
        Ok(mut expr) => {
            // If the user passed a top-level struct literal (e.g., `obj!(Board { ... })`),
            // automatically add `.. ::snow_ui::default()` when there is no `..rest`.
            // This keeps the change conservative and avoids rewriting nested macros/expressions.
            if let syn::Expr::Struct(es) = &mut expr {
                // Try to convert into a small defaulting block for known core types to avoid
                // requiring per-field `..default()` in user code and to safely handle commas.
                // Build assignment list for named fields.
                let path = &es.path;
                let assigns: Vec<proc_macro2::TokenStream> = Vec::new();
                let ok = true;
                // Walk nested expressions inside each field to add `..default()` when
                // we encounter nested struct literals (e.g., `Board { ... }`).
                fn add_defaults_to_expr(e: &mut syn::Expr) {
                    match e {
                        syn::Expr::Struct(es) => {
                            // First recurse into fields so nested struct literals inside
                            // these fields are also default-augmented.
                            for field in es.fields.iter_mut() {
                                add_defaults_to_expr(&mut field.expr);
                            }

                            // If this is a `Form { submit_handler: ... }` literal and the
                            // assigned expression is a bare path (function name), wrap it
                            // as `Box::new(...)` so the handler can be stored as a trait object.
                            if es.path.segments.last().map(|s| s.ident == "Form").unwrap_or(false) {
                                for field in es.fields.iter_mut() {
                                    if let syn::Member::Named(ident) = &field.member {
                                        if ident == "submit_handler" {
                                            if let syn::Expr::Path(_) = &field.expr {
                                                let orig = &field.expr;
                                                field.expr = syn::parse2(quote! { std::sync::Arc::new(|form: &::snow_ui::Form| Box::pin({ let __owned = form.clone(); async move { (#orig)(&__owned).await } })) }).expect("failed to wrap submit_handler");
                                            }
                                        }
                                    }
                                }
                            }

                            // If there is no `..rest` on this nested struct, rebuild the
                            // struct literal with an explicit comma before `..` to avoid
                            // it being parsed as a range (`a..b`).
                            if es.rest.is_none() {
                                let path = &es.path;
                                let fields_tokens: Vec<proc_macro2::TokenStream> =
                                    es.fields.iter().map(|f| quote! { #f }).collect();
                                *e = syn::parse2(quote! { #path { #(#fields_tokens),* , .. ::snow_ui::prelude::default() } }).expect("failed to rebuild nested struct with defaults");
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
                        syn::Expr::Paren(p) => add_defaults_to_expr(&mut *p.expr),
                        syn::Expr::Reference(r) => add_defaults_to_expr(&mut *r.expr),
                        syn::Expr::Block(b) => {
                            for stmt in b.block.stmts.iter_mut() {
                                if let syn::Stmt::Expr(expr, _) = stmt {
                                    add_defaults_to_expr(expr);
                                }
                            }
                        }
                        syn::Expr::Unary(u) => add_defaults_to_expr(&mut *u.expr),
                        syn::Expr::Binary(b) => {
                            add_defaults_to_expr(&mut *b.left);
                            add_defaults_to_expr(&mut *b.right);
                        }
                        _ => {}
                    }
                }

                for field in es.fields.iter_mut() {
                    add_defaults_to_expr(&mut field.expr);
                }

                if ok && !assigns.is_empty() {
                    let block = quote! {{
                        // Construct by calling `Default::default()` on the type. The `#[element]`
                        // macro emits an `impl Default` helper for elements so this should
                        // succeed for both builtin and element types.
                        let mut __tmp: #path = ::std::default::Default::default();
                        #(#assigns)*
                        __tmp
                    }};
                    expr = syn::parse2(block).expect("failed to build defaulting block");
                } else if es.rest.is_none() {
                    // Rebuild the struct literal token-stream ensuring there's a comma
                    // before the `..` so it doesn't parse as a range (e.g. `a..b`).
                    let path = &es.path;
                    let fields: Vec<proc_macro2::TokenStream> =
                        es.fields.iter().map(|f| quote! { #f }).collect();
                    let rebuilt =
                        quote! { #path { #(#fields),* , .. ::snow_ui::prelude::default() } };
                    expr = syn::parse2(rebuilt).expect("failed to build nested defaulting struct");
                }
            }

            let expanded = quote! {
                ::snow_ui_macros::__list_item!(#expr).into()
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
pub fn element(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse optional `message = [Type1, Type2]` (preferred), or `message = "Type1, Type2"`,
    // or legacy `register = ...`. Use `syn` to parse structured forms when possible.
    let mut message_paths: Vec<syn::Path> = Vec::new();
    if !attr.is_empty() {
        // Fallback to simple string parsing of the attribute tokens to support
        // forms like `message = [A, B]`, `message = "A,B"`, or `register = ...`.
        let s = attr.to_string();
        for key in ["message", "register"] {
            let mut start = 0usize;
            while let Some(pos) = s[start..].find(key) {
                let idx = start + pos;
                // Find '=' after the key
                if let Some(eq_pos) = s[idx..].find('=') {
                    let after_eq = idx + eq_pos + 1;
                    let rest = s[after_eq..].trim_start();
                    if rest.starts_with('[') {
                        if let Some(end) = rest.find(']') {
                            let inner = &rest[1..end];
                            for part in inner.split(',') {
                                let p = part.trim();
                                if !p.is_empty() {
                                    if let Ok(path) = syn::parse_str::<syn::Path>(p) {
                                        message_paths.push(path);
                                    }
                                }
                            }
                        }
                    } else if rest.starts_with('"') {
                        if let Some(end) = rest[1..].find('"') {
                            let inner = &rest[1..1 + end];
                            for part in inner.split(',') {
                                let p = part.trim();
                                if !p.is_empty() {
                                    if let Ok(path) = syn::parse_str::<syn::Path>(p) {
                                        message_paths.push(path);
                                    }
                                }
                            }
                        }
                    } else {
                        // single path or comma-separated without brackets
                        let mut token = String::new();
                        for c in rest.chars() {
                            if c == ',' || c == ')' || c == ']' {
                                break;
                            }
                            token.push(c);
                        }
                        let token = token.trim();
                        if !token.is_empty() {
                            for part in token.split(',') {
                                let p = part.trim();
                                if !p.is_empty() {
                                    if let Ok(path) = syn::parse_str::<syn::Path>(p) {
                                        message_paths.push(path);
                                    }
                                }
                            }
                        }
                    }
                }
                start = idx + key.len();
            }
        }
    }

    // Parse the item as a struct and generate the same helpful `IntoObject` impl as before.
    // If `message_paths` is non-empty, use explicit registration for those message types.
    // If `message_paths` is empty, use inventory-based auto-registration via `register_handlers_for_instance`.
    match syn::parse::<syn::ItemStruct>(item.clone()) {
        Ok(s) => {
            // Ensure a `Default` impl exists for element structs so `obj!` can auto-default them.
            let name = &s.ident;
            // Check for `#[derive(Default)]` on the struct; if absent, we'll prepend one when emitting.
            let mut has_default = false;
            for attr in &s.attrs {
                if attr.path().is_ident("derive") {
                    // Inspect the attribute's meta list tokens and look for `Default` to avoid
                    // depending on nested-meta APIs that differ across versions.
                    if let syn::Meta::List(list) = &attr.meta {
                        let tok = list.tokens.to_string();
                        if tok.contains("Default") {
                            has_default = true;
                            break;
                        }
                    }
                }
            }
            // Emit the struct item unchanged; we'll add any helper impls separately so
            // we don't modify the user's original attributes.
            let struct_item = quote! { #s };

            // Generate a hidden default factory and an `impl Default` (when missing)
            // so callers like `obj!` can create a default instance without requiring
            // the user to add `#[derive(Default)]` themselves.
            let mut default_impl = quote! {};
            // Only generate when there are no generic parameters (simpler and safe for now)
            if s.generics.params.is_empty() {
                match &s.fields {
                    syn::Fields::Named(n) if !n.named.is_empty() => {
                        let assigns = n.named.iter().map(|f| {
                            let ident = &f.ident;
                            quote! { #ident: ::std::default::Default::default() }
                        });
                        default_impl = quote! {
                            impl #name {
                                #[doc(hidden)]
                                fn __snow_ui_default() -> Self {
                                    #name { #(#assigns),* }
                                }
                            }
                        };
                    }
                    syn::Fields::Unnamed(u) if !u.unnamed.is_empty() => {
                        let defaults = (0..u.unnamed.len())
                            .map(|_| quote! { ::std::default::Default::default() });
                        default_impl = quote! {
                            impl #name {
                                #[doc(hidden)]
                                fn __snow_ui_default() -> Self {
                                    #name( #(#defaults),* )
                                }
                            }
                        };
                    }
                    syn::Fields::Unit => {
                        default_impl = quote! {
                            impl #name {
                                #[doc(hidden)]
                                fn __snow_ui_default() -> Self { #name }
                            }
                        };
                    }
                    _ => {}
                }

                // If the original struct didn't have a `Default` derive, also provide an `impl Default` that uses the factory.
                if !has_default {
                    default_impl = quote! {
                        #default_impl
                        impl ::std::default::Default for #name {
                            fn default() -> Self { #name::__snow_ui_default() }
                        }
                    };
                }
            }

            match &s.fields {
                syn::Fields::Unnamed(u) if u.unnamed.len() == 1 => {
                    let field_ty = &u.unnamed.iter().next().unwrap().ty;
                    let is_button = if let syn::Type::Path(p) = field_ty {
                        p.path.segments.last().unwrap().ident == "Button"
                    } else {
                        false
                    };

                    if is_button {
                        if message_paths.is_empty() {
                            // Use inventory-based auto-registration
                            quote! {
                                #struct_item
                                #default_impl
                                impl ::snow_ui::IntoObject for #name {
                                    fn into_object(self) -> ::snow_ui::Object {
                                        if ::snow_ui::has_registered_handlers::<#name>() {
                                            let rc = ::std::rc::Rc::new(::std::cell::RefCell::new(self));
                                            ::snow_ui::register_handlers_for_instance(&rc);
                                            let e: ::snow_ui::Element = rc.borrow().0.clone().into();
                                            e.into()
                                        } else {
                                            let e: ::snow_ui::Element = self.0.into();
                                            e.into()
                                        }
                                    }
                                }
                            }
                            .into()
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
                                        let e: ::snow_ui::Element = rc.borrow().0.clone().into();
                                        e.into()
                                    }
                                }
                            }
                            .into()
                        }
                    } else {
                        if message_paths.is_empty() {
                            // Use inventory-based auto-registration
                            quote! {
                                #struct_item
                                #default_impl
                                impl ::snow_ui::IntoObject for #name {
                                    fn into_object(self) -> ::snow_ui::Object {
                                        if ::snow_ui::has_registered_handlers::<#name>() {
                                            let rc = ::std::rc::Rc::new(::std::cell::RefCell::new(self));
                                            ::snow_ui::register_handlers_for_instance(&rc);
                                            rc.borrow().0.clone().into()
                                        } else {
                                            self.0.into()
                                        }
                                    }
                                }
                            }
                            .into()
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
                                        rc.borrow().0.into()
                                    }
                                }
                            }
                            .into()
                        }
                    }
                }
                syn::Fields::Named(n) if n.named.len() == 1 => {
                    let field = n.named.iter().next().unwrap();
                    let field_ident = field.ident.as_ref().unwrap();
                    let field_ty = &field.ty;
                    let is_button = if let syn::Type::Path(p) = field_ty {
                        p.path.segments.last().unwrap().ident == "Button"
                    } else {
                        false
                    };

                    if is_button {
                        if message_paths.is_empty() {
                            // Use inventory-based auto-registration
                            quote! {
                                #struct_item
                                #default_impl
                                impl ::snow_ui::IntoObject for #name {
                                    fn into_object(self) -> ::snow_ui::Object {
                                        if ::snow_ui::has_registered_handlers::<#name>() {
                                            let rc = ::std::rc::Rc::new(::std::cell::RefCell::new(self));
                                            ::snow_ui::register_handlers_for_instance(&rc);
                                            let e: ::snow_ui::Element = rc.borrow().#field_ident.clone().into();
                                            e.into()
                                        } else {
                                            let e: ::snow_ui::Element = self.#field_ident.into();
                                            e.into()
                                        }
                                    }
                                }
                            }
                            .into()
                        } else {
                            let regs = message_paths.iter();
                            quote! {
                                #struct_item
                                impl ::snow_ui::IntoObject for #name {
                                    fn into_object(self) -> ::snow_ui::Object {
                                        let rc = ::std::rc::Rc::new(::std::cell::RefCell::new(self));
                                        #(
                                            ::snow_ui::event_bus().register_handler::<#name, #regs>(rc.clone());
                                        )*
                                        let e: ::snow_ui::Element = rc.borrow().#field_ident.clone().into();
                                        e.into()
                                    }
                                }
                            }
                            .into()
                        }
                    } else {
                        if message_paths.is_empty() {
                            // Use inventory-based auto-registration
                            quote! {
                                #struct_item
                                #default_impl
                                impl ::snow_ui::IntoObject for #name {
                                    fn into_object(self) -> ::snow_ui::Object {
                                        if ::snow_ui::has_registered_handlers::<#name>() {
                                            let rc = ::std::rc::Rc::new(::std::cell::RefCell::new(self));
                                            ::snow_ui::register_handlers_for_instance(&rc);
                                            rc.borrow().#field_ident.clone().into()
                                        } else {
                                            self.#field_ident.into()
                                        }
                                    }
                                }
                            }
                            .into()
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
                                        rc.borrow().#field_ident.clone().into()
                                    }
                                }
                            }
                            .into()
                        }
                    }
                }
                // Handle structs with multiple named fields or no fields - use inventory auto-registration
                _ => {
                    quote! {
                        #struct_item
                        #default_impl
                        impl ::snow_ui::IntoObject for #name {
                            fn into_object(self) -> ::snow_ui::Object {
                                if ::snow_ui::has_registered_handlers::<#name>() {
                                    let rc = ::std::rc::Rc::new(::std::cell::RefCell::new(self));
                                    ::snow_ui::register_handlers_for_instance(&rc);
                                    // For complex structs, we just return a placeholder
                                    // The actual conversion should be customized
                                    unimplemented!(concat!("IntoObject not fully implemented for ", stringify!(#name), " - consider adding a custom From impl"));
                                } else {
                                    unimplemented!(concat!("IntoObject not implemented for ", stringify!(#name)));
                                }
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
