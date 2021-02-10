use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

mod attr;
mod enum_impl;
mod fn_impl;
mod parsers;
mod struct_impl;
mod ty;

#[proc_macro_derive(Parse, attributes(nommy))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let span = input.span();
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
                match struct_impl::Named::new(ident, generics, attrs, fields) {
                    Ok(t) => t.into_token_stream(),
                    Err(e) => e.to_compile_error(),
                }
            }
            syn::Fields::Unnamed(fields) => {
                match struct_impl::Unnamed::new(ident, generics, attrs, fields) {
                    Ok(t) => t.into_token_stream(),
                    Err(e) => e.to_compile_error(),
                }
            }
            syn::Fields::Unit => match struct_impl::Unit::new(ident, generics, attrs) {
                Ok(t) => t.into_token_stream(),
                Err(e) => e.to_compile_error(),
            },
        },
        syn::Data::Enum(enum_data) => {
            match enum_impl::Enum::new(ident, generics, attrs, enum_data) {
                Ok(t) => t.into_token_stream(),
                Err(e) => e.to_compile_error(),
            }
        }
        syn::Data::Union(_) => syn::Error::new(span, "unions not supported").into_compile_error(),
    }
    .into()
}
