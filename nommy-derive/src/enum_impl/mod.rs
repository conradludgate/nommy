mod named;
mod unit;
mod unnamed;

use named::EnumVariantNamed;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use unit::EnumVariantUnit;
use unnamed::EnumVariantUnnamed;

use crate::{attr::GlobalAttr, fn_impl::{parse_or, BuildOutput, FnImpl}, parsers::{FunctionBuilder, Parser, Peeker}};

pub struct Enum {
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub variants: Vec<EnumVariant>,
    generic: syn::Type,
}

fn map<T, U, F: FnMut(&T) -> U>(input: &Vec<T>, f: F) -> Vec<U> {
    input.iter().map(f).collect()
}
fn map_unzip<T, U1, U2, F: FnMut(&T) -> (U1, U2)>(input: &Vec<T>, f: F) -> (Vec<U1>, Vec<U2>) {
    input.iter().map(f).unzip()
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

        let error_message = format!("no variants of {} could be parsed", name);
        let var_names = map(
            &vars,
            |v| map_vars!(v => |n| n.name.to_string().to_lowercase()),
        );
        let var_names_parse = map(&var_names, |name| format_ident!("__parse_{}", name));
        let var_names_peek = map(&var_names, |name| format_ident!("__peek_{}", name));

        let (variant_parse_impls, variant_parse_wc) = map_unzip(vars, |v| {
            let BuildOutput { fn_impl, wc } = map_vars! {v => |n| (n, self).build_parse()};
            (fn_impl, wc)
        });
        let mut parse_wc: Vec<_> = variant_parse_wc.iter().flatten().cloned().collect();

        let (variant_peek_impls, variant_peek_wc) = map_unzip(vars, |v| {
            let BuildOutput { fn_impl, wc } = map_vars! {v => |n| (n, self).build_peek()};
            (fn_impl, wc)
        });
        let mut peek_wc: Vec<_> = variant_peek_wc.iter().flatten().cloned().collect();

        let mut parse_builder =
            FunctionBuilder::<Parser>::new(&mut parse_wc, generic, &attrs.ignore_whitespace);
        let parse_prefix = parse_builder.fix(&attrs.prefix, "prefix", name.to_string());
        let parse_suffix = parse_builder.fix(&attrs.suffix, "suffix", name.to_string());

        let mut peek_builder =
            FunctionBuilder::<Peeker>::new(&mut peek_wc, generic, &attrs.ignore_whitespace);
        let peek_prefix = peek_builder.fix(&attrs.prefix, "prefix", name.to_string());
        let peek_suffix = peek_builder.fix(&attrs.suffix, "suffix", name.to_string());

        tokens.extend(quote!{
            #[automatically_derived]
            impl <#generic: Clone, #(#args),*> ::nommy::Peek<#generic> for #name<#(#args),*>
            where #(
                #peek_wc: ::nommy::Peek<#generic>,
            )* {
                fn peek(input: &mut impl ::nommy::Buffer<#generic>) -> bool {
                    #peek_prefix

                    if #( !Self::#var_names_peek(&mut input.cursor()) )&&* {
                        return false;
                    }

                    #peek_suffix

                    true
                }
            }

            #[automatically_derived]
            impl <#generic: Clone, #(#args),*> ::nommy::Parse<#generic> for #name<#(#args),*>
            where #(
                #parse_wc: ::nommy::Parse<#generic>,
            )* {
                fn parse(input: &mut impl ::nommy::Buffer<#generic>) -> ::nommy::eyre::Result<Self> {
                    use ::nommy::eyre::WrapErr;

                    #parse_prefix

                    let result = #(
                        if Self::#var_names_peek(&mut input.cursor()) {
                            Self::#var_names_parse(input).wrap_err("variant peek succeeded but parse failed")
                        } else
                    )* {
                        Err(::nommy::eyre::eyre!(#error_message))
                    }?;

                    #parse_suffix

                    Ok(result)
                }
            }

            #[automatically_derived]
            impl<#(#args),*> #name<#(#args),*>
            {
                #(
                    fn #var_names_parse<#generic> (input: &mut impl ::nommy::Buffer<#generic>) -> ::nommy::eyre::Result<Self>
                    where #(
                        #variant_parse_wc: ::nommy::Parse<#generic>,
                    )* {
                        use ::nommy::eyre::WrapErr;
                        #variant_parse_impls
                    }

                    fn #var_names_peek<#generic: Clone> (input: &mut impl ::nommy::Buffer<#generic>) -> bool
                    where #(
                        #variant_peek_wc: ::nommy::Peek<#generic>,
                    )* {
                        #variant_peek_impls
                        true
                    }
                )*
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
    ) -> Self {
        let args = generics.type_params().cloned().map(|tp| tp.ident).collect();

        let variants = enum_data
            .variants
            .into_iter()
            .map(|v| match v.fields {
                syn::Fields::Named(named) => EnumVariant::Named(EnumVariantNamed {
                    name: v.ident,
                    attrs: GlobalAttr::parse_attrs(v.attrs),
                    fields: named.named.into_iter().map(|f| f.into()).collect(),
                }),
                syn::Fields::Unnamed(unnamed) => EnumVariant::Unnamed(EnumVariantUnnamed {
                    name: v.ident,
                    attrs: GlobalAttr::parse_attrs(v.attrs),
                    fields: unnamed.unnamed.into_iter().map(|f| f.into()).collect(),
                }),
                syn::Fields::Unit => EnumVariant::Unit(EnumVariantUnit {
                    name: v.ident,
                    attrs: GlobalAttr::parse_attrs(v.attrs),
                }),
            })
            .collect();

        let attrs = GlobalAttr::parse_attrs(attrs);
        let generic = parse_or(&attrs.parse_type);

        Enum {
            name,
            attrs,
            args,
            variants,
            generic,
        }
    }
}

pub enum EnumVariant {
    Named(EnumVariantNamed),
    Unnamed(EnumVariantUnnamed),
    Unit(EnumVariantUnit),
}
