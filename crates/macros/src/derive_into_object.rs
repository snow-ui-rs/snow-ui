//! Implementation of `#[derive(IntoObject)]`.

use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub(crate) fn derive(input: DeriveInput) -> proc_macro2::TokenStream {
    let name = input.ident;

    // Parse optional helper attribute: `#[into_object(expr = "...")]` or `#[into_object(field = "...")]`
    let mut attr_expr: Option<syn::LitStr> = None;
    let mut attr_field: Option<syn::Ident> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("into_object")
            && let syn::Meta::List(list) = &attr.meta
        {
            let tokens_string = list.tokens.to_string();
            if let Some(idx) = tokens_string.find("expr")
                && let Some(q1) = tokens_string[idx..].find('"')
            {
                let rest = &tokens_string[idx + q1 + 1..];
                if let Some(q2) = rest.find('"') {
                    let val = &rest[..q2];
                    attr_expr = Some(syn::LitStr::new(val, proc_macro2::Span::call_site()));
                }
            }
            if let Some(idx) = tokens_string.find("field")
                && let Some(q1) = tokens_string[idx..].find('"')
            {
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

    match input.data {
        Data::Struct(ref s) => match &s.fields {
            // User provided an `expr` override
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
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                gen_unnamed_single_field(&name, &fields.unnamed.iter().next().unwrap().ty)
            }
            // User provided a `field` override for a named struct
            Fields::Named(fields) if attr_field.is_some() => {
                let chosen = attr_field.as_ref().unwrap();
                let actual_field = fields
                    .named
                    .iter()
                    .find(|f| f.ident.as_ref().map(|i| i == chosen).unwrap_or(false))
                    .expect("specified field not found");
                gen_named_field_conversion(&name, chosen, &actual_field.ty)
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
    }
}

/// Generate `IntoObject` for a tuple struct with a single unnamed field.
fn gen_unnamed_single_field(name: &syn::Ident, field_ty: &syn::Type) -> proc_macro2::TokenStream {
    if let syn::Type::Reference(r) = field_ty {
        return gen_ref_conversion(name, quote!(self.0), r);
    }
    if let syn::Type::Path(p) = field_ty
        && p.path.segments.last().unwrap().ident == "String"
    {
        return quote! {
            impl ::snow_ui::IntoObject for #name {
                fn into_object(self) -> ::snow_ui::Object {
                    self.0.into()
                }
            }
        };
    }
    // Fallback
    quote! {
        impl ::snow_ui::IntoObject for #name {
            fn into_object(self) -> ::snow_ui::Object {
                self.0.into()
            }
        }
    }
}

/// Generate `IntoObject` for a named struct field.
fn gen_named_field_conversion(
    name: &syn::Ident,
    field_ident: &syn::Ident,
    field_ty: &syn::Type,
) -> proc_macro2::TokenStream {
    if let syn::Type::Reference(r) = field_ty {
        return gen_ref_conversion(name, quote!(self.#field_ident), r);
    }
    if let syn::Type::Path(p) = field_ty
        && p.path.segments.last().unwrap().ident == "String"
    {
        return quote! {
            impl ::snow_ui::IntoObject for #name {
                fn into_object(self) -> ::snow_ui::Object {
                    self.#field_ident.into()
                }
            }
        };
    }
    // Fallback
    quote! {
        impl ::snow_ui::IntoObject for #name {
            fn into_object(self) -> ::snow_ui::Object {
                self.#field_ident.into()
            }
        }
    }
}

/// Generate conversion for a reference type field (handles `&'static str` vs non-static).
fn gen_ref_conversion(
    name: &syn::Ident,
    accessor: proc_macro2::TokenStream,
    r: &syn::TypeReference,
) -> proc_macro2::TokenStream {
    let is_static = r
        .lifetime
        .as_ref()
        .map(|lt| lt.ident == "static")
        .unwrap_or(false);

    if is_static {
        quote! {
            impl ::snow_ui::IntoObject for #name {
                fn into_object(self) -> ::snow_ui::Object {
                    ::snow_ui::Text { text: #accessor, .. ::snow_ui::default() }.into()
                }
            }
        }
    } else {
        quote! {
            impl ::snow_ui::IntoObject for #name {
                fn into_object(self) -> ::snow_ui::Object {
                    let s: &'static str = Box::leak(#accessor.to_owned().into_boxed_str());
                    ::snow_ui::Text { text: s, .. ::snow_ui::default() }.into()
                }
            }
        }
    }
}
