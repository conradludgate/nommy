use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::attr::FieldAttr;

use super::named_struct::{NamedField};

pub struct Args(pub Vec<syn::Ident>);

impl ToTokens for Args {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.0.is_empty() {
            return;
        }
        let args = &self.0;

        tokens.extend(quote! {
            < #( #args ),* >
        })
    }
}

pub struct Enum {
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub fields: Vec<EnumField>,
}

impl Enum {
    pub fn new(derive: &syn::DeriveInput, enum_data: &syn::DataEnum) -> Self {
        let name = derive.ident.clone();

        let args = derive
            .generics
            .type_params()
            .cloned()
            .map(|tp| tp.ident)
            .collect();

        let fields = enum_data
            .variants
            .iter()
            .map(|v| match &v.fields {
                syn::Fields::Named(named) => EnumField {
                    name: v.ident.clone(),
                    field_type: EnumFieldType::Named(
                        named
                            .named
                            .iter()
                            .cloned()
                            .map(|field| {
                                let mut attrs = FieldAttr::default();
                                for attr in field.attrs {
                                    if attr.path.is_ident("nommy") {
                                        attrs.parse_attr(attr.tokens.clone());
                                    }
                                }
                                NamedField {
                                    attrs,
                                    name: field.ident.unwrap(),
                                    ty: field.ty,
                                }
                            })
                            .collect(),
                    ),
                },
                syn::Fields::Unnamed(tuple) => EnumField {
                    name: v.ident.clone(),
                    field_type: EnumFieldType::Tuple(
                        tuple.unnamed.iter().map(|field| field.ty.clone()).collect(),
                    ),
                },
                syn::Fields::Unit => panic!("Unit variants not supported in enum parse derive"),
            })
            .collect();

        Enum { name, args, fields }
    }
}

impl ToTokens for Enum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Enum { name, args, fields } = self;

        let peek_impl = EnumPeek {
            name: name.clone(),
            args: args.clone(),
            fields: fields.clone(),
        };

        let parse_impl = EnumParse {
            name: name.clone(),
            args: args.clone(),
            fields: fields.clone(),
        };

        tokens.extend(quote! {
            #peek_impl

            #parse_impl
        })
    }
}

#[derive(Clone)]
pub struct EnumField {
    pub name: syn::Ident,
    pub field_type: EnumFieldType,
}

#[derive(Clone)]
pub enum EnumFieldType {
    // None, // not supported
    Tuple(Vec<syn::Type>),
    Named(Vec<NamedField>),
}

struct EnumPeek {
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub fields: Vec<EnumField>,
}

impl ToTokens for EnumPeek {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let EnumPeek { name, args, fields } = self;

        let mut impl_args = args.clone();
        let generic_ident = format_ident!("__PeekType");
        impl_args.push(generic_ident.clone());
        let impl_args = Args(impl_args);

        let args = Args(args.clone());

        let peek_where_clause = fields
            .iter()
            .flat_map(|field| match &field.field_type {
                EnumFieldType::Tuple(v) => v.clone(),
                EnumFieldType::Named(named) => named.iter().map(|field| field.ty.clone()).collect(),
            })
            .map(|ty| {
                quote! {#ty: ::nommy::Peek<#generic_ident>}
            });

        let peek_rows = fields.iter().map(|field| match &field.field_type {
            EnumFieldType::Tuple(t) => {
                quote! {
                    let mut cursor = input.cursor();
                    if #(<#t as ::nommy::Peek<#generic_ident>>::peek(&mut cursor))&&* {
                        let step = cursor.close();
                        input.fast_forward(step);
                        return true;
                    }
                }
            }
            EnumFieldType::Named(n) => {
                let t = n.iter().map(|f| &f.ty);
                quote! {
                    let mut cursor = input.cursor();
                    if #(<#t as ::nommy::Peek<#generic_ident>>::peek(&mut cursor))&&* {
                        let step = cursor.close();
                        input.fast_forward(step);
                        return true;
                    }
                }
            }
        });

        tokens.extend(quote! {
            #[automatically_derived]
            impl #impl_args ::nommy::Peek<#generic_ident> for #name #args
            where #(
                #peek_where_clause,
            )* {
                fn peek(input: &mut ::nommy::Cursor<impl ::std::iter::Iterator<Item=#generic_ident>>) -> bool {
                    #( #peek_rows )*

                    false
                }
            }
        })
    }
}

struct EnumParse {
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub fields: Vec<EnumField>,
}

impl ToTokens for EnumParse {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let EnumParse { name, args, fields } = self;

        let mut impl_args = args.clone();
        let generic_ident = format_ident!("__ParseType");
        impl_args.push(generic_ident.clone());
        let impl_args = Args(impl_args);

        let args = Args(args.clone());

        let parse_where_clause = fields
            .iter()
            .flat_map(|field| match &field.field_type {
                EnumFieldType::Tuple(v) => v.clone(),
                EnumFieldType::Named(named) => named.iter().map(|field| field.ty.clone()).collect(),
            })
            .map(|ty| {
                quote! {#ty: ::nommy::Parse<#generic_ident>}
            });

        let parse_rows = fields.iter().map(|field| {
            let field_name = &field.name;
            match &field.field_type {
                EnumFieldType::Tuple(t) => {
                    quote!{
                        let mut cursor = input.cursor();
                        if #(<#t as ::nommy::Peek<#generic_ident>>::peek(&mut cursor))&&* {
                            return Ok(#name::#field_name(
                                #(<#t as ::nommy::Parse<#generic_ident>>::parse(input)
                                    .map_err(|_| ::nommy::impls::EnumParseError)?,)*
                            ));
                        }
                    }
                }
                EnumFieldType::Named(n) => {
                    let t = n.iter().map(|f|&f.ty);
                    let sets = n.iter().map(|f| {
                        let NamedField { attrs: _, name, ty } = f;
                        quote!{
                            #name: <#ty as ::nommy::Parse<#generic_ident>>::parse(input)
                                .map_err(|_| ::nommy::impls::EnumParseError)?
                        }
                    });

                    quote!{
                        let mut cursor = input.cursor();
                        if #(<#t as ::nommy::Peek<#generic_ident>>::peek(&mut cursor))&&* {
                            return Ok(#name::#field_name{
                                #( #sets, )*
                            })
                        }
                    }
                }
            }
        });

        let error_message = format!("no variants of {} could be parsed", name);

        tokens.extend(quote! {
            #[automatically_derived]
            impl #impl_args ::nommy::Parse<#generic_ident> for #name #args
            where #(
                #parse_where_clause,
            )* {
                fn parse(input: &mut ::nommy::Buffer<impl Iterator<Item = #generic_ident>>) -> ::nommy::eyre::Result<Self> {
                    #( #parse_rows )*

                    Err(::nommy::eyre::eyre!(#error_message))
                }
            }
        })
    }
}
