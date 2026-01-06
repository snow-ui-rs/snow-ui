use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(IntoWidget)]
pub fn derive_into_widget(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = match input.data {
        Data::Struct(ref s) => match &s.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                quote! {
                    impl ::snow_ui::IntoWidget for #name {
                        fn into_widget(self) -> ::snow_ui::Widget {
                            self.0.into()
                        }
                    }
                }
            }
            Fields::Named(fields) if fields.named.len() == 1 => {
                let field_name = &fields.named.iter().next().unwrap().ident;
                quote! {
                    impl ::snow_ui::IntoWidget for #name {
                        fn into_widget(self) -> ::snow_ui::Widget {
                            self.#field_name.into()
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
