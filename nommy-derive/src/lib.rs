use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

mod enum_impl;
mod named_struct;
mod attr;

use enum_impl::Enum;
use named_struct::NamedStruct;

#[proc_macro_derive(Parse, attributes(nommy))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(named) => NamedStruct::new(&input, named).into_token_stream(),
            syn::Fields::Unnamed(_) =>  unimplemented!(),
            syn::Fields::Unit => unimplemented!(),
        },
        syn::Data::Enum(e) => Enum::new(&input, e).into_token_stream(),
        syn::Data::Union(_) => panic!("unions not supported"),
    }
    .into()
}
