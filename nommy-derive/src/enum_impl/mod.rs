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

        let mut outer_builder = Builder::new(generic, name, &attrs.parse_type);

        outer_builder.create_ignore(&attrs.ignore);
        outer_builder.add_fix(&attrs.prefix, "prefix", format!("enum `{}`", name));
        outer_builder.start_variants();

        for v in vars {
            let BuildOutput {
                peek_impl,
                parse_impl,
                wc,
            } = map_vars! {v => |n| n.fn_impl(&self).build(&name)};

            outer_builder.add_where_raw(wc.clone());

            let var_name = map_vars!(v => |n| n.name.to_string().to_lowercase());
            let parse_name = format_ident!("__parse_{}", var_name);
            let peek_name = format_ident!("__peek_{}", var_name);
            let parse_result = map_vars!(v => |n| n.result(&self));

            outer_builder.add_variant(&peek_name, &parse_name);



            let (peek_fn, parse_fn) = match attrs.parse_type {
                Some(_) => (
                    quote!{
                        fn #peek_name(input: &mut impl ::nommy::Buffer<#generic>) -> bool
                    },
                    quote!{
                        fn #parse_name(input: &mut impl ::nommy::Buffer<#generic>) -> ::nommy::eyre::Result<Self>
                    },
                ),
                None => (
                    quote!{
                        fn #peek_name<#generic>(input: &mut impl ::nommy::Buffer<#generic>) -> bool where #wc
                    },
                    quote!{
                        fn #parse_name<#generic>(input: &mut impl ::nommy::Buffer<#generic>) -> ::nommy::eyre::Result<Self> where #wc
                    },
                ),
            };

            tokens.extend(quote!{
                #[automatically_derived]
                impl<#(#args),*> #name<#(#args),*>
                {
                    #parse_fn {
                        use ::nommy::eyre::WrapErr;
                        use ::std::convert::TryInto;
                        #parse_impl
                        #parse_result
                    }

                    #peek_fn {
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
                type Args = ();
                fn parse(input: &mut impl ::nommy::Buffer<#generic>, _: &()) -> ::nommy::eyre::Result<Self> {
                    use ::nommy::eyre::WrapErr;
                    use ::std::convert::TryInto;
                    #parse_impl

                    Ok(result)
                }

                fn peek(input: &mut impl ::nommy::Buffer<#generic>, _: &()) -> bool {
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
        let attrs = GlobalAttr::parse_attrs(attrs)?;
        let generic = parse_or(&attrs.parse_type);

        let variants = enum_data
            .variants
            .into_iter()
            .map(|v| match v.fields {
                syn::Fields::Named(named) => Ok(EnumVariant::Named(EnumVariantNamed {
                    name: v.ident,
                    attrs: GlobalAttr::parse_attrs(v.attrs)?.extend_with(&attrs),
                    fields: named
                        .named
                        .into_iter()
                        .map(|f| f.try_into())
                        .collect::<syn::Result<_>>()?,
                })),
                syn::Fields::Unnamed(unnamed) => Ok(EnumVariant::Unnamed(EnumVariantUnnamed {
                    name: v.ident,
                    attrs: GlobalAttr::parse_attrs(v.attrs)?.extend_with(&attrs),
                    fields: unnamed
                        .unnamed
                        .into_iter()
                        .map(|f| f.try_into())
                        .collect::<syn::Result<_>>()?,
                })),
                syn::Fields::Unit => Ok(EnumVariant::Unit(EnumVariantUnit {
                    name: v.ident,
                    attrs: GlobalAttr::parse_attrs(v.attrs)?.extend_with(&attrs),
                })),
            })
            .collect::<syn::Result<_>>()?;

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
