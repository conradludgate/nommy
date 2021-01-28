use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::named_struct::{Args, NamedField};

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
                            .map(|field| NamedField {
                                name: field.ident.clone().unwrap(),
                                ty: field.ty.clone(),
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
                        let NamedField { name, ty } = f;
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

        tokens.extend(quote! {
            #[automatically_derived]
            impl #impl_args ::nommy::Parse<#generic_ident> for #name #args
            where #(
                #parse_where_clause,
            )* {
                type Error = ::nommy::impls::EnumParseError;
                fn parse(input: &mut ::nommy::Buffer<impl Iterator<Item = #generic_ident>>) -> Result<Self, Self::Error> {
                    #( #parse_rows )*

                    Err(::nommy::impls::EnumParseError)
                }
            }
        })
    }
}

// impl<T> Peek<T> for Foo
// where
//     crate::text::token::Dot: Peek<T>,
//     crate::text::token::LParen: Peek<T>,
//     crate::text::token::RParen: Peek<T>,
// {
//     fn peek(input: &mut Cursor<impl Iterator<Item = #generic_ident>>) -> bool {
//         let mut cursor = input.cursor();
//         if crate::text::token::Dot::peek(&mut cursor) {
//             let step = cursor.close();
//             input.fast_forward(step);
//             return true;
//         }

//         let mut cursor = input.cursor();
//         if crate::text::token::LParen::peek(&mut cursor)
//             && crate::text::token::RParen::peek(&mut cursor)
//         {
//             let step = cursor.close();
//             input.fast_forward(step);
//             return true;
//         }

//         false
//     }
// }

// impl<T> Parse<T> for Foo
// where
//     crate::text::token::Dot: Parse<T>,
//     crate::text::token::LParen: Parse<T>,
//     crate::text::token::RParen: Parse<T>,
// {
//     type Error = EnumParseError;
//     fn parse(input: &mut Buffer<impl Iterator<Item = #generic_ident>>) -> Result<Self, Self::Error> {
//         let mut cursor = input.cursor();
//         if crate::text::token::Dot::peek(&mut cursor) {
//             return Ok(Foo::Bar(
//                 crate::text::token::Dot::parse(input).map_err(|_| EnumParseError)?,
//             ));
//         }

//         let mut cursor = input.cursor();
//         if crate::text::token::LParen::peek(&mut cursor)
//             && crate::text::token::RParen::peek(&mut cursor)
//         {
//             return Ok(Foo::Baz{
//                 foo: crate::text::token::LParen::parse(input).map_err(|_| EnumParseError)?,
//                 bar: crate::text::token::RParen::parse(input).map_err(|_| EnumParseError)?,
//             });
//         }

//         Err(EnumParseError)
//     }
// }
