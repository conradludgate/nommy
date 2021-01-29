use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::attr::{FieldAttr, GlobalAttr, IgnoreWS};

pub struct NamedStruct {
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub fields: Vec<NamedField>,
}

impl NamedStruct {
    pub fn new(derive: &syn::DeriveInput, syn_fields: &syn::FieldsNamed) -> Self {
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
            .map(|field| {
                let mut attrs = FieldAttr::default();
                for attr in &field.attrs {
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
            .collect();

        let mut attrs = GlobalAttr::default();
        for attr in &derive.attrs {
            if attr.path.is_ident("nommy") {
                attrs.parse_attr(attr.tokens.clone());
            }
        }

        NamedStruct {
            attrs,
            name,
            args,
            fields,
        }
    }
}

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

impl ToTokens for NamedStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NamedStruct {
            attrs,
            name,
            args,
            fields,
        } = self;

        let peek_impl = NamedStructPeek {
            attrs: attrs.clone(),
            name: name.clone(),
            args: args.clone(),
            fields: fields.clone(),
        };

        let parse_impl = NamedStructParse {
            attrs: attrs.clone(),
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

#[derive(Debug, Clone)]
pub struct NamedField {
    pub attrs: FieldAttr,
    pub name: syn::Ident,
    pub ty: syn::Type,
}

pub struct NamedStructPeek {
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub fields: Vec<NamedField>,
}

impl ToTokens for NamedStructPeek {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NamedStructPeek { attrs, name, args, fields } = self;

        let mut impl_args = args.clone();
        let generic_ident = format_ident!("__PeekType");
        impl_args.push(generic_ident.clone());
        let impl_args = Args(impl_args);

        let args = Args(args.clone());

        let mut where_clause_types = vec![];

        let ignore_whitespace = match &attrs.ignore_whitespace {
            Some(IgnoreWS::Spaces) => {
                let ty: syn::Type =
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::Space>}).unwrap();
                where_clause_types.push(ty.clone());
                quote! {
                    <#ty as ::nommy::Peek<#generic_ident>>::peek(input) &&
                }
            }
            Some(IgnoreWS::All) => {
                let ty: syn::Type =
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::WhiteSpace>}).unwrap();
                where_clause_types.push(ty.clone());
                quote! {
                    <#ty as ::nommy::Peek<#generic_ident>>::peek(input) &&
                }
            }
            None => quote! {},
        };

        let peek_rows = fields.iter().enumerate().map(|(i, field)| {
            let NamedField { attrs, name: _, ty } = field;

            let mut tokens = TokenStream::new();

            if i > 0 {
                tokens.extend(ignore_whitespace.clone());
            }

            if let Some(prefix) = &attrs.prefix {
                where_clause_types.push(prefix.clone());
                tokens.extend(quote! {
                    <#prefix as ::nommy::Peek<#generic_ident>>::peek(input) &&
                    #ignore_whitespace
                });
            };

            match &attrs.parser {
                Some(ty) => {
                    where_clause_types.push(ty.clone());
                    tokens.extend(quote! {
                        <#ty as ::nommy::Peek<#generic_ident>>::peek(input) &&
                    });
                },
                None => {
                    where_clause_types.push(ty.clone());
                    tokens.extend(quote! {
                        <#ty as ::nommy::Peek<#generic_ident>>::peek(input) &&
                    });
                },
            };

            if let Some(suffix) = &attrs.suffix {
                where_clause_types.push(suffix.clone());
                tokens.extend(quote! {
                    #ignore_whitespace
                    <#suffix as ::nommy::Peek<#generic_ident>>::peek(input) &&
                });
            };

            tokens
        }).collect::<Vec<_>>();

        let prefix = match &attrs.prefix {
            Some(prefix) => {
                where_clause_types.push(prefix.clone());
                quote! {
                    <#prefix as ::nommy::Peek<#generic_ident>>::peek(input) &&
                    #ignore_whitespace
                }
            }
            None => quote! {},
        };

        let suffix = match &attrs.suffix {
            Some(suffix) => {
                where_clause_types.push(suffix.clone());
                quote! {
                    #ignore_whitespace
                    <#suffix as ::nommy::Peek<#generic_ident>>::peek(input) &&
                }
            }
            None => quote! {},
        };

        let peek_where_clause = where_clause_types
            .iter()
            .map(|ty| quote! {#ty: ::nommy::Peek<#generic_ident>});

        tokens.extend(quote! {
            #[automatically_derived]
            impl #impl_args ::nommy::Peek<#generic_ident> for #name #args
            where #(
                #peek_where_clause,
            )* {
                fn peek(input: &mut ::nommy::Cursor<impl ::std::iter::Iterator<Item=#generic_ident>>) -> bool {
                    #prefix

                    #( #peek_rows )*

                    #suffix

                    true
                }
            }
        })
    }
}

pub struct NamedStructParse {
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub fields: Vec<NamedField>,
}

impl ToTokens for NamedStructParse {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NamedStructParse {
            attrs,
            name,
            args,
            fields,
        } = self;

        let mut impl_args = args.clone();
        let generic_ident = format_ident!("__ParseType");
        impl_args.push(generic_ident.clone());
        let impl_args = Args(impl_args);

        let args = Args(args.clone());

        let mut where_clause_types = vec![];

        let ignore_whitespace = match &attrs.ignore_whitespace {
            Some(IgnoreWS::Spaces) => {
                let ty: syn::Type =
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::Space>}).unwrap();
                where_clause_types.push(ty.clone());
                quote! {
                    <#ty as ::nommy::Parse<#generic_ident>>::parse(input).expect("parsing spaces should not fail, but did");
                }
            }
            Some(IgnoreWS::All) => {
                let ty: syn::Type =
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::WhiteSpace>}).unwrap();
                where_clause_types.push(ty.clone());
                quote! {
                    <#ty as ::nommy::Parse<#generic_ident>>::parse(input).expect("parsing whitespace should not fail, but did");
                }
            }
            None => quote! {},
        };

        let parse_rows = fields.iter().enumerate().map(|(i, field)| {
            let NamedField { attrs, name, ty } = field;
            let error = format!("could not parse field `{}`", name);

            let mut tokens = TokenStream::new();

            if i > 0 {
                tokens.extend(ignore_whitespace.clone());
            }

            if let Some(prefix) = &attrs.prefix {
                where_clause_types.push(prefix.clone());
                let error = format!("could not parse prefix for field `{}`", name);
                tokens.extend(quote! {
                    <#prefix as ::nommy::Parse<#generic_ident>>::parse(input).wrap_err(#error)?;
                    #ignore_whitespace
                });
            };

            match &attrs.parser {
                Some(ty) => {
                    where_clause_types.push(ty.clone());
                    tokens.extend(quote! {
                        let #name = <#ty as ::nommy::Parse<#generic_ident>>::parse(input).wrap_err(#error)?.process();
                    });
                },
                None => {
                    where_clause_types.push(ty.clone());
                    tokens.extend(quote! {
                        let #name = <#ty as ::nommy::Parse<#generic_ident>>::parse(input).wrap_err(#error)?;
                    });
                },
            };

            if let Some(suffix) = &attrs.suffix {
                where_clause_types.push(suffix.clone());
                let error = format!("could not parse suffix for field `{}`", name);
                tokens.extend(quote! {
                    #ignore_whitespace
                    <#suffix as ::nommy::Parse<#generic_ident>>::parse(input).wrap_err(#error)?;
                });
            };

            tokens
        }).collect::<Vec<_>>();

        let create_output_rows = fields.iter().map(|field| {
            let name = &field.name;
            quote! {
                #name: #name.into(),
            }
        });

        let prefix = match &attrs.prefix {
            Some(prefix) => {
                let error = format!("could not parse prefix for struct `{}`", name);
                where_clause_types.push(prefix.clone());
                quote! {
                    <#prefix as ::nommy::Parse<#generic_ident>>::parse(input).wrap_err(#error)?;
                    #ignore_whitespace
                }
            }
            None => quote! {},
        };

        let suffix = match &attrs.suffix {
            Some(suffix) => {
                let error = format!("could not parse suffix for  `{}`", name);
                where_clause_types.push(suffix.clone());
                quote! {
                    #ignore_whitespace
                    <#suffix as ::nommy::Parse<#generic_ident>>::parse(input).wrap_err(#error)?;
                }
            }
            None => quote! {},
        };

        let parse_where_clause = where_clause_types
            .iter()
            .map(|ty| quote! {#ty: ::nommy::Parse<#generic_ident>});

        tokens.extend(quote! {
            #[automatically_derived]
            impl #impl_args ::nommy::Parse<#generic_ident> for #name #args
            where #(
                #parse_where_clause,
            )* {
                fn parse(input: &mut ::nommy::Buffer<impl ::std::iter::Iterator<Item=#generic_ident>>) -> ::nommy::eyre::Result<Self> {
                    use ::nommy::eyre::WrapErr;

                    #prefix

                    #( #parse_rows )*

                    #suffix

                    Ok(#name{
                        #( #create_output_rows )*
                    })
                }
            }
        })
    }
}
