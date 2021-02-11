use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    attr::GlobalAttr,
    fn_impl::{parse_or, BuildOutput, FnImpl},
    parsers::NamedField,
};

pub struct Unit {
    name: syn::Ident,
    args: Vec<syn::Ident>,
    attrs: GlobalAttr,
    generic: syn::Type,
}

impl ToTokens for Unit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Unit {
            name,
            args,
            attrs,
            generic,
        } = self;

        let fields: &[NamedField] = &[];
        let fn_impl = FnImpl {
            ty: "struct",
            name,
            fields,
            attrs,
            generic,
        };

        let BuildOutput {
            peek_impl,
            parse_impl,
            wc,
        } = fn_impl.build(&name);

        let impl_line = match attrs.parse_type {
            Some(_) => quote!{
                impl<#(#args),*> ::nommy::Parse<#generic> for #name<#(#args),*>
            },
            None => quote!{
                impl<#generic, #(#args),*> ::nommy::Parse<#generic> for #name<#(#args),*> where #wc
            },
        };

        tokens.extend(quote!{
            #[automatically_derived]
            #impl_line {
                fn parse(input: &mut impl ::nommy::Buffer<#generic>) -> ::nommy::eyre::Result<Self> {
                    use ::nommy::eyre::WrapErr;
                    #parse_impl
                    Ok(#name)
                }

                fn peek(input: &mut impl ::nommy::Buffer<#generic>) -> bool {
                    #peek_impl
                    true
                }
            }
        })
    }
}

impl Unit {
    pub fn new(
        name: syn::Ident,
        generics: syn::Generics,
        attrs: Vec<syn::Attribute>,
    ) -> syn::Result<Self> {
        let args = generics.type_params().cloned().map(|tp| tp.ident).collect();
        let attrs = GlobalAttr::parse_attrs(attrs)?;
        let generic = parse_or(&attrs.parse_type);

        Ok(Unit {
            attrs,
            name,
            args,
            generic,
        })
    }
}
