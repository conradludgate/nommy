use proc_macro::TokenStream;
use quote::{ToTokens};
use syn::{parse_macro_input, DeriveInput};

mod named_struct;
mod util;
use named_struct::NamedStruct;


#[proc_macro_derive(Parse)]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(named) => {
                NamedStruct::new(&input, named).into_token_stream()
            }
            syn::Fields::Unnamed(fields) => {
                eprintln!("struct unnamed fields: {:?}", fields.unnamed);
                unimplemented!()
            }
            syn::Fields::Unit => {
                eprintln!("struct unit");
                unimplemented!()
            }
        },
        syn::Data::Enum(e) => {
            eprintln!("enum variants: {:?}", e.variants);
            unimplemented!()
        }
        syn::Data::Union(_) => panic!("unions not supported"),
    }.into()
}
