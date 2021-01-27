use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};

pub struct NamedStruct {
    pub vis: syn::Visibility,
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub fields: Vec<NamedField>,
}

impl NamedStruct {
    pub fn new(derive: &syn::DeriveInput, syn_fields: &syn::FieldsNamed) -> Self {
        let vis = derive.vis.clone();
        let name = derive.ident.clone();

        let args = derive
            .generics
            .type_params()
            .cloned()
            .map(|tp| tp.ident)
            .collect();

        let fields = syn_fields
            .named
            .clone()
            .into_iter()
            .map(|field| NamedField {
                name: field.ident.unwrap(),
                ty: field.ty,
            })
            .collect();

        NamedStruct {
            vis,
            name,
            args,
            fields,
        }
    }
}

struct Args(Vec<syn::Ident>);

impl ToTokens for Args {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if self.0.len() == 0 {
            return;
        }
        let args = &self.0;

        tokens.extend(quote! {
            < #( #args ),* >
        })
    }
}

use heck::CamelCase;

impl ToTokens for NamedStruct {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let NamedStruct {
            vis,
            name,
            args,
            fields,
        } = self;

        let error_name = format_ident!("{}ParseError", name);

        let enum_error = EnumError {
            vis: vis.clone(),
            name: error_name.clone(),
            field_names: fields.iter().map(|field| field.name.clone()).collect(),
        };

        let peek_impl = NamedStructPeek {
            name: name.clone(),
            args: args.clone(),
            fields: fields.clone(),
        };

        let parse_impl = NamedStructParse {
            name: name.clone(),
            args: args.clone(),
            fields: fields.clone(),
            error_name,
        };

        tokens.extend(quote! {
            #enum_error

            #peek_impl

            #parse_impl
        })
    }
}

#[derive(Debug, Clone)]
pub struct NamedField {
    pub name: syn::Ident,
    pub ty: syn::Type,
}

pub struct EnumError {
    pub vis: syn::Visibility,
    pub name: syn::Ident,
    pub field_names: Vec<syn::Ident>,
}

impl ToTokens for EnumError {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let EnumError {
            vis,
            name,
            field_names,
        } = self;

        let field_type_args = field_names
            .iter()
            .map(|field_name| format_ident!("{}Error", field_name.to_string().to_camel_case()))
            .collect::<Vec<_>>();

        let field_type_where_clauses = field_names
            .iter()
            .map(|field_name| {
                let ty = format_ident!("{}Error", field_name.to_string().to_camel_case());
                quote! {#ty: ::std::error::Error}
            })
            .collect::<Vec<_>>();

        let fields = field_names.iter().map(|field_name| {
            let camel = format_ident!("{}", field_name.to_string().to_camel_case());
            let ty = format_ident!("{}Error", camel);
            quote! {#camel(::std::boxed::Box<#ty>)}
        });

        let error_display_rows = field_names.iter().map(|field_name| {
            let camel = format_ident!("{}", field_name.to_string().to_camel_case());
            let error_message = format!("could not parse field `{}`: {{}}", field_name.to_string());
            quote! { #name::#camel(_0) => write!(__formatter, #error_message, _0) }
        });

        tokens.extend(quote! {
            #[derive(Debug, PartialEq)]
            #vis enum #name <#(#field_type_args),*> where #(#field_type_where_clauses),* {
                #( #fields, )*
            }
            impl<#(#field_type_args),*> ::std::error::Error for #name <#(#field_type_args),*> where #(#field_type_where_clauses),* {}
            #[automatically_derived]
            impl<#(#field_type_args),*> ::std::fmt::Display for #name <#(#field_type_args),*> where #(#field_type_where_clauses),* {
                fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match self {
                        #( #error_display_rows, )*
                    }
                }
            }
        })
    }
}

pub struct NamedStructPeek {
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub fields: Vec<NamedField>,
}

impl ToTokens for NamedStructPeek {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let NamedStructPeek { name, args, fields } = self;

        let mut impl_args = args.clone();
        let generic_ident = format_ident!("__T");
        impl_args.push(generic_ident.clone());
        let impl_args = Args(impl_args);

        let args = Args(args.clone());

        let peek_where_clause = fields.iter().map(|field| {
            let ty = &field.ty;
            quote! {#ty: ::nommy::Peek<#generic_ident>}
        });

        let peek_rows = fields.iter().map(|field| {
            let ty = &field.ty;
            quote! { <#ty as ::nommy::Peek<#generic_ident>>::peek(input) }
        });

        tokens.extend(quote! {
            #[automatically_derived]
            impl #impl_args ::nommy::Peek<#generic_ident> for #name #args
            where #(
                #peek_where_clause,
            )* {
                fn peek(input: &mut ::nommy::Cursor<impl ::std::iter::Iterator<Item=#generic_ident>>) -> bool {
                    #( #peek_rows )&&*
                }
            }
        })
    }
}

pub struct NamedStructParse {
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub fields: Vec<NamedField>,
    pub error_name: syn::Ident,
}

impl ToTokens for NamedStructParse {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let NamedStructParse {
            name,
            args,
            fields,
            error_name,
        } = self;

        let mut impl_args = args.clone();
        let generic_ident = format_ident!("__T");
        impl_args.push(generic_ident.clone());
        let impl_args = Args(impl_args);

        let args = Args(args.clone());

        let parse_where_clause = fields.iter().map(|field| {
            let ty = &field.ty;
            quote! {#ty: ::nommy::Parse<#generic_ident>}
        });

        let error_type_args = fields
            .iter()
            .map(|field| {
                let ty = &field.ty;
                quote! { < #ty as ::nommy::Parse<#generic_ident>>::Error }
            })
            .collect::<Vec<_>>();

        let parse_rows = fields
            .iter()
            .map(|field| {
                let NamedField { name, ty } = field;
                let camel = format_ident!("{}", name.to_string().to_camel_case());
                quote!{
                    #name: <#ty as ::nommy::Parse<#generic_ident>>::parse(input)
                        .map_err(|err| #error_name::< #( #error_type_args ),* >::#camel(Box::new(err)))?
                }
            })
            .collect::<Vec<_>>();

        tokens.extend(quote! {
            #[automatically_derived]
            impl #impl_args ::nommy::Parse<#generic_ident> for #name #args
            where #(
                #parse_where_clause,
            )* {
                type Error = #error_name < #( #error_type_args ),* >;

                fn parse(input: &mut ::nommy::Buffer<impl ::std::iter::Iterator<Item=#generic_ident>>) -> Result<Self, Self::Error> {
                    Ok(#name{
                        #( #parse_rows, )*
                    })
                    //
                }
            }
        })
    }
}
