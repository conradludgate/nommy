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

impl FnImpl<NamedField> for Unit {
    const TYPE: &'static str = "struct";
    fn fields(&self) -> &[NamedField] {
        &[]
    }
    fn name(&self) -> &syn::Ident {
        &self.name
    }
    fn generic(&self) -> &syn::Type {
        &self.generic
    }
    fn attrs(&self) -> &GlobalAttr {
        &self.attrs
    }
    fn result(&self) -> TokenStream {
        let name = &self.name;
        quote! {
            Ok(#name)
        }
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

impl ToTokens for Unit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Unit {
            name,
            args,
            attrs: _,
            generic,
        } = self;
        let BuildOutput {
            fn_impl: parse_fn_impl,
            wc: parse_wc,
        } = self.build_parse();
        let BuildOutput {
            fn_impl: peek_fn_impl,
            wc: peek_wc,
        } = self.build_peek();

        tokens.extend(quote!{
            #[automatically_derived]
            impl<#generic, #(#args),*> ::nommy::Parse<#generic> for #name<#(#args),*>
            where #parse_wc {
                fn parse(input: &mut impl ::nommy::Buffer<#generic>) -> ::nommy::eyre::Result<Self> {
                    use ::nommy::eyre::WrapErr;
                    #parse_fn_impl
                }
            }

            #[automatically_derived]
            impl<#generic, #(#args),*> ::nommy::Peek<#generic> for #name<#(#args),*>
            where #peek_wc {
                fn peek(input: &mut impl ::nommy::Buffer<#generic>) -> bool {
                    #peek_fn_impl
                    true
                }
            }
        })
    }
}
