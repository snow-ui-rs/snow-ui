//! Implementation of `#[derive(Message)]` and `#[message]` attribute macro.

use quote::quote;
use syn::{Data, DeriveInput};

/// Logic for `#[derive(Message)]`.
pub(crate) fn derive(input: DeriveInput) -> proc_macro2::TokenStream {
    let name = input.ident;

    match input.data {
        Data::Struct(_) => {
            quote! {
                impl ::snow_ui::Message for #name {}
            }
        }
        _ => quote! {
            compile_error!("Message can only be derived for structs");
        },
    }
}

/// Logic for the `#[message]` attribute macro.
pub(crate) fn attribute(item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    match syn::parse2::<syn::ItemStruct>(item) {
        Ok(s) => {
            let name = &s.ident;
            quote! {
                #s
                impl ::snow_ui::Message for #name {}
            }
        }
        Err(e) => e.to_compile_error(),
    }
}
