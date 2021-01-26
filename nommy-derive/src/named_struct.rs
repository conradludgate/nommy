use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};

use crate::util::type_contains_generic;

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

struct ParseRow {
    pub error: syn::Ident,
    pub field: NamedField,
}

use heck::CamelCase;
impl ToTokens for ParseRow {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ParseRow { error, field } = self;
        let NamedField { name, ty } = field;

        let pascal_name = syn::Ident::new(
            &name.to_string().to_camel_case(),
            proc_macro2::Span::call_site(),
        );

        tokens.extend(quote! {
            let (#name, input) = <#ty as ::nommy::Parse>::parse(input)
                .map_err(|err| #error::#pascal_name(Box::new(err)))?;
        })
    }
}

struct ErrorRow {
    pub field: NamedField,
}

impl ToTokens for ErrorRow {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let NamedField { name, ty } = &self.field;
        let pascal_name = syn::Ident::new(
            &name.to_string().to_camel_case(),
            proc_macro2::Span::call_site(),
        );
        let error_message = format!("could not parse field `{}`: {{0:?}}", name.to_string());

        tokens.extend(quote! {
            #[error(#error_message)]
            #pascal_name(Box<<#ty as Parse>::Error>),
        })
    }
}

struct WhereClause {
    pub clauses: Vec<Clause>,
}

impl ToTokens for WhereClause {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if self.clauses.len() == 0 {
            return;
        }
        let clauses = &self.clauses;

        tokens.extend(quote! {
            where #( #clauses ),*
        })
    }
}

struct Clause {
    pub ty: syn::Type,
}

impl ToTokens for Clause {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ty = &self.ty;
        tokens.extend(quote! {
            #ty: ::nommy::Parse
        })
    }
}

impl ToTokens for NamedStruct {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let NamedStruct {
            vis,
            name,
            args,
            fields,
        } = self;

        let mut where_clause = WhereClause { clauses: vec![] };
        for field in fields {
            if type_contains_generic(&field.ty, args) {
                where_clause.clauses.push(Clause {
                    ty: field.ty.clone(),
                });
            }
        }

        let error = format_ident!("{}Err", name);
        let args = Args(args.clone());

        let error_rows = fields.iter().map(|field| ErrorRow {
            field: field.clone(),
        });

        let parse_rows = fields.iter().map(|field| ParseRow {
            error: error.clone(),
            field: field.clone(),
        });

        let field_names = fields.iter().map(|field| field.name.clone());

        tokens.extend(quote! {
            #[derive(Debug, ::nommy::thiserror::Error)]
            #vis enum #error #args #where_clause {
                #( #error_rows )*
            }

            impl #args ::nommy::Parse for #name #args #where_clause {
                type Error = #error #args;
                fn parse(input: &str) -> ::std::result::Result<(Self, &str), Self::Error> {
                    #( #parse_rows )*

                    Ok((#name { #( #field_names ),* }, input))
                }
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct NamedField {
    pub name: syn::Ident,
    pub ty: syn::Type,
}
