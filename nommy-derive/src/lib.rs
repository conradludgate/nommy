use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

mod attr;
mod fn_impl;
mod parsers;
mod enum_impl;
mod struct_impl;

#[proc_macro_derive(Parse, attributes(nommy))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let DeriveInput {
        attrs,
        vis: _,
        ident,
        generics,
        data,
    } = input;

    match data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(fields) => {
                struct_impl::Named::new(ident, generics, attrs, fields).into_token_stream()
            }
            syn::Fields::Unnamed(fields) => {
                struct_impl::Unnamed::new(ident, generics, attrs, fields).into_token_stream()
            }
            syn::Fields::Unit => struct_impl::Unit::new(ident, generics, attrs).into_token_stream(),
        },
        syn::Data::Enum(enum_data) => {
            enum_impl::Enum::new(ident, generics, attrs, enum_data).into_token_stream()
        }
        syn::Data::Union(_) => panic!("unions not supported"),
    }
    .into()
}
