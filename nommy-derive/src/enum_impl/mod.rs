mod named;
mod unit;
mod unnamed;

use std::convert::TryInto;

use named::EnumVariantNamed;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use unit::EnumVariantUnit;
use unnamed::EnumVariantUnnamed;

use crate::{
    attr::GlobalAttr,
    fn_impl::{parse_or, BuildOutput, Builder},
};

pub struct Enum {
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub variants: Vec<EnumVariant>,
    generic: syn::Type,
}

macro_rules! map_vars {
    ($variant:expr => |$v:ident| $expr:expr) => {
        match $variant {
            EnumVariant::Named($v) => $expr,
            EnumVariant::Unnamed($v) => $expr,
            EnumVariant::Unit($v) => $expr,
        }
    };
}

impl ToTokens for Enum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Enum {
            attrs,
            name,
            args,
            variants: vars,
            generic,
        } = self;

        let mut outer_builder = Builder::new(generic, name);

        outer_builder.create_ignore(&attrs.ignore);
        outer_builder.add_fix(&attrs.prefix, "prefix", format!("enum `{}`", name));
        outer_builder.start_variants();

        for v in vars {
            let BuildOutput {
                peek_impl,
                parse_impl,
                wc,
            } = map_vars! {v => |n| n.fn_impl(&self).build()};

            outer_builder.add_where_raw(wc.clone());

            let var_name = map_vars!(v => |n| n.name.to_string().to_lowercase());
            let parse_name = format_ident!("__parse_{}", var_name);
            let peek_name = format_ident!("__peek_{}", var_name);
            let parse_result = map_vars!(v => |n| n.result(&self));

            outer_builder.add_variant(&peek_name, &parse_name);

            tokens.extend(quote!{
                #[automatically_derived]
                impl<#(#args),*> #name<#(#args),*>
                {
                    fn #parse_name<#generic> (input: &mut impl ::nommy::Buffer<#generic>) -> ::nommy::eyre::Result<Self> where #wc {
                        use ::nommy::eyre::WrapErr;
                        #parse_impl
                        #parse_result
                    }

                    fn #peek_name<#generic> (input: &mut impl ::nommy::Buffer<#generic>) -> bool where #wc {
                        #peek_impl

                        true
                    }
                }
            })
        }

        outer_builder.finish_variants(format!("no variants of {} could be parsed", name));

        outer_builder.add_fix(&attrs.suffix, "suffix", format!("enum `{}`", name));

        let BuildOutput {
            peek_impl,
            parse_impl,
            wc,
        } = outer_builder.build();

        tokens.extend(quote!{
            #[automatically_derived]
            impl <#generic, #(#args),*> ::nommy::Parse<#generic> for #name<#(#args),*>
            where #wc {
                fn parse(input: &mut impl ::nommy::Buffer<#generic>) -> ::nommy::eyre::Result<Self> {
                    use ::nommy::eyre::WrapErr;
                    #parse_impl

                    Ok(result)
                }

                fn peek(input: &mut impl ::nommy::Buffer<#generic>) -> bool {
                    #peek_impl

                    true
                }
            }
        })
    }
}

impl Enum {
    pub fn new(
        name: syn::Ident,
        generics: syn::Generics,
        attrs: Vec<syn::Attribute>,
        enum_data: syn::DataEnum,
    ) -> syn::Result<Self> {
        let args = generics.type_params().cloned().map(|tp| tp.ident).collect();

        let variants = enum_data
            .variants
            .into_iter()
            .map(|v| match v.fields {
                syn::Fields::Named(named) => Ok(EnumVariant::Named(EnumVariantNamed {
                    name: v.ident,
                    attrs: GlobalAttr::parse_attrs(v.attrs)?,
                    fields: named
                        .named
                        .into_iter()
                        .map(|f| f.try_into())
                        .collect::<syn::Result<_>>()?,
                })),
                syn::Fields::Unnamed(unnamed) => Ok(EnumVariant::Unnamed(EnumVariantUnnamed {
                    name: v.ident,
                    attrs: GlobalAttr::parse_attrs(v.attrs)?,
                    fields: unnamed
                        .unnamed
                        .into_iter()
                        .map(|f| f.try_into())
                        .collect::<syn::Result<_>>()?,
                })),
                syn::Fields::Unit => Ok(EnumVariant::Unit(EnumVariantUnit {
                    name: v.ident,
                    attrs: GlobalAttr::parse_attrs(v.attrs)?,
                })),
            })
            .collect::<syn::Result<_>>()?;

        let attrs = GlobalAttr::parse_attrs(attrs)?;
        let generic = parse_or(&attrs.parse_type);

        Ok(Enum {
            name,
            attrs,
            args,
            variants,
            generic,
        })
    }
}

pub enum EnumVariant {
    Named(EnumVariantNamed),
    Unnamed(EnumVariantUnnamed),
    Unit(EnumVariantUnit),
}
