use std::convert::TryInto;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    attr::GlobalAttr,
    fn_impl::{parse_or, BuildOutput, FnImpl},
    parsers::NamedField,
};

pub struct Named {
    name: syn::Ident,
    fields: Vec<NamedField>,
    args: Vec<syn::Ident>,
    attrs: GlobalAttr,
    generic: syn::Type,
}

impl FnImpl<NamedField> for Named {
    const TYPE: &'static str = "struct";
    fn fields(&self) -> &[NamedField] {
        &self.fields
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
}

impl Named {
    pub fn new(
        name: syn::Ident,
        generics: syn::Generics,
        attrs: Vec<syn::Attribute>,
        fields: syn::FieldsNamed,
    ) -> syn::Result<Self> {
        let args = generics.type_params().cloned().map(|tp| tp.ident).collect();
        let fields = fields
            .named
            .into_iter()
            .map(|f| f.try_into())
            .collect::<syn::Result<_>>()?;
        let attrs = GlobalAttr::parse_attrs(attrs)?;
        let generic = parse_or(&attrs.parse_type);

        Ok(Named {
            attrs,
            name,
            args,
            fields,
            generic,
        })
    }
}

impl ToTokens for Named {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Named {
            name,
            fields: _,
            args,
            attrs: _,
            generic,
        } = self;

        let BuildOutput {
            peek_impl,
            parse_impl,
            wc,
        } = self.build();

        let names = self.fields.iter().map(|f| &f.name);

        tokens.extend(quote!{
            #[automatically_derived]
            impl<#generic, #(#args),*> ::nommy::Parse<#generic> for #name<#(#args),*>
            where #wc {
                fn parse(input: &mut impl ::nommy::Buffer<#generic>) -> ::nommy::eyre::Result<Self> {
                    use ::nommy::eyre::WrapErr;
                    #parse_impl

                    Ok(#name {#(
                        #names,
                    )*})
                }

                fn peek(input: &mut impl ::nommy::Buffer<#generic>) -> bool {
                    #peek_impl
                    true
                }
            }
        });
    }
}
//         let impl_args = match (attrs.debug, &attrs.parse_type) {
//             (_, Some(_)) => quote!{ <#(#args),*> },
//             (true, None) => quote!{ <#peek_type: Clone + ::std::fmt::Debug, #(#args),*> },
//             (false, None) => quote!{ <#peek_type, #(#args),*> },
//         };

//         let debug = match attrs.debug {
//             true => {
//                 let struct_name = name.to_string();
//                 quote!{
//                     println!("peeking `{}` with input starting with {:?}", #struct_name, input.cursor().collect::<Vec<_>>());
//                 }
//             }
//             _ => quote!{},
//         };
