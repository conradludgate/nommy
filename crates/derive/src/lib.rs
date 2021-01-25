use proc_macro::TokenStream;
use syn::{parse_quote, parse_macro_input};
use quote::ToTokens;

#[proc_macro_derive(Parse)]
pub fn derive_parse(attr: TokenStream, input: TokenStream) -> TokenStream {

}
