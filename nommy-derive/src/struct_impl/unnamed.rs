use std::convert::TryInto;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    attr::GlobalAttr,
    fn_impl::{parse_or, BuildOutput, FnImpl},
    parsers::{FieldType, UnnamedField},
};

pub struct Unnamed {
    name: syn::Ident,
    fields: Vec<UnnamedField>,
    args: Vec<syn::Ident>,
    attrs: GlobalAttr,
    generic: syn::Type,
}

impl ToTokens for Unnamed {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Unnamed {
            name,
            args,
            generic,
            fields,
            attrs,
        } = self;

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

        let names = self.fields.iter().enumerate().map(|(i, f)| f.name(i));

        tokens.extend(quote!{
            #[automatically_derived]
            #impl_line {
                fn parse(input: &mut impl ::nommy::Buffer<#generic>) -> ::nommy::eyre::Result<Self> {
                    use ::nommy::eyre::WrapErr;
                    use ::std::convert::TryInto;
                    #parse_impl

                    Ok(#name (#(
                        #names,
                    )*))
                }

                fn peek(input: &mut impl ::nommy::Buffer<#generic>) -> bool {
                    #peek_impl
                    true
                }
            }
        })
    }
}

impl Unnamed {
    pub fn new(
        name: syn::Ident,
        generics: syn::Generics,
        attrs: Vec<syn::Attribute>,
        fields: syn::FieldsUnnamed,
    ) -> syn::Result<Self> {
        let args = generics.type_params().cloned().map(|tp| tp.ident).collect();
        let fields = fields
            .unnamed
            .into_iter()
            .map(|f| f.try_into())
            .collect::<syn::Result<_>>()?;
        let attrs = GlobalAttr::parse_attrs(attrs)?;
        let generic = parse_or(&attrs.parse_type);

        Ok(Unnamed {
            attrs,
            name,
            args,
            fields,
            generic,
        })
    }
}
