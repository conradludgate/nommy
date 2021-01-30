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
            peek_type: format_ident!("__PeekType"),
        };

        let parse_impl = NamedStructParse {
            attrs: attrs.clone(),
            name: name.clone(),
            args: args.clone(),
            fields: fields.clone(),
            parse_type: format_ident!("__ParseType"),
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
    pub peek_type: syn::Ident,
}

impl NamedStructPeek {
    fn peek(&self, peeker: syn::Type) -> Peek {
        Peek {
            peeker,
            peek_type: self.peek_type.clone(),
        }
    }
}

impl ToTokens for NamedStructPeek {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NamedStructPeek {
            attrs,
            name,
            args,
            fields,
            peek_type,
        } = self;

        let mut impl_args = args.clone();
        impl_args.push(peek_type.clone());
        let impl_args = Args(impl_args);

        let args = Args(args.clone());

        let mut where_clause_types = vec![];

        let ignore_whitespace = match &attrs.ignore_whitespace {
            Some(IgnoreWS::Spaces) => {
                let ty: syn::Type =
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::Space>}).unwrap();
                where_clause_types.push(ty.clone());
                self.peek(ty).into_token_stream()
            }
            Some(IgnoreWS::All) => {
                let ty: syn::Type =
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::WhiteSpace>}).unwrap();
                where_clause_types.push(ty.clone());
                self.peek(ty).into_token_stream()
            }
            None => Default::default(),
        };

        let peek_rows = fields
            .iter()
            .map(|field| {
                let NamedField { attrs, name: _, ty } = field;

                let mut tokens = TokenStream::new();

                if let Some(prefix) = &attrs.prefix {
                    where_clause_types.push(prefix.clone());
                    self.peek(prefix.clone()).to_tokens(&mut tokens);
                    tokens.extend(ignore_whitespace.clone());
                };

                let parser = attrs.parser.clone().unwrap_or(ty.clone());
                where_clause_types.push(parser.clone());

                self.peek(parser).to_tokens(&mut tokens);
                tokens.extend(ignore_whitespace.clone());

                if let Some(suffix) = &attrs.suffix {
                    where_clause_types.push(suffix.clone());
                    self.peek(suffix.clone()).to_tokens(&mut tokens);
                    tokens.extend(ignore_whitespace.clone());
                };

                tokens
            })
            .collect::<Vec<_>>();

        let prefix = match &attrs.prefix {
            Some(prefix) => {
                where_clause_types.push(prefix.clone());
                let mut tokens = self.peek(prefix.clone()).to_token_stream();
                tokens.extend(ignore_whitespace.clone());
                tokens
            }
            None => Default::default(),
        };

        let suffix = match &attrs.suffix {
            Some(suffix) => {
                where_clause_types.push(suffix.clone());
                let mut tokens = self.peek(suffix.clone()).to_token_stream();
                tokens.extend(ignore_whitespace.clone());
                tokens
            }
            None => Default::default(),
        };

        let peek_where_clause = where_clause_types
            .iter()
            .map(|ty| quote! {#ty: ::nommy::Peek<#peek_type>});

        tokens.extend(quote! {
            #[automatically_derived]
            impl #impl_args ::nommy::Peek<#peek_type> for #name #args
            where #(
                #peek_where_clause,
            )* {
                fn peek(input: &mut ::nommy::Cursor<impl ::std::iter::Iterator<Item=#peek_type>>) -> bool {
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
    pub parse_type: syn::Ident,
}

impl NamedStructParse {
    fn parse(&self, parser: syn::Type, error: String) -> Parse {
        Parse {
            parser,
            error,
            parse_type: self.parse_type.clone(),
            process: false,
        }
    }
}

impl ToTokens for NamedStructParse {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NamedStructParse {
            attrs,
            name,
            args,
            fields,
            parse_type,
        } = self;

        let mut impl_args = args.clone();
        impl_args.push(parse_type.clone());
        let impl_args = Args(impl_args);

        let args = Args(args.clone());

        let mut where_clause_types = vec![];

        let ignore_whitespace = match &attrs.ignore_whitespace {
            Some(IgnoreWS::Spaces) => {
                let ty: syn::Type =
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::Space>}).unwrap();
                where_clause_types.push(ty.clone());

                self.parse(ty, "parsing spaces should not fail, but did".to_string())
                    .into_token_stream()
            }
            Some(IgnoreWS::All) => {
                let ty: syn::Type =
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::WhiteSpace>}).unwrap();
                where_clause_types.push(ty.clone());
                self.parse(
                    ty,
                    "parsing whitespace should not fail, but did".to_string(),
                )
                .into_token_stream()
            }
            None => Default::default(),
        };

        let parse_rows = fields
            .iter()
            .map(|field| {
                let NamedField { attrs, name, ty } = field;

                let mut tokens = TokenStream::new();

                if let Some(prefix) = &attrs.prefix {
                    where_clause_types.push(prefix.clone());
                    self.parse(
                        prefix.clone(),
                        format!("could not parse prefix for field `{}`", name),
                    )
                    .to_tokens(&mut tokens);
                    tokens.extend(ignore_whitespace.clone());
                };

                let parser = attrs.parser.clone().unwrap_or(ty.clone());
                where_clause_types.push(parser.clone());

                tokens.extend(quote! {let #name = });
                self.parse(parser.clone(), format!("could not parse field `{}`", name))
                    .with_process(attrs.parser.is_some())
                    .to_tokens(&mut tokens);
                tokens.extend(ignore_whitespace.clone());

                if let Some(suffix) = &attrs.suffix {
                    where_clause_types.push(suffix.clone());

                    self.parse(
                        suffix.clone(),
                        format!("could not parse suffix for field `{}`", name),
                    )
                    .to_tokens(&mut tokens);
                    tokens.extend(ignore_whitespace.clone());
                };

                tokens
            })
            .collect::<Vec<_>>();

        let create_output_rows = fields.iter().map(|field| {
            let name = &field.name;
            quote! {
                #name: #name.into(),
            }
        });

        let prefix = attrs.prefix.clone().map_or(Default::default(), |prefix| {
            where_clause_types.push(prefix.clone());

            let mut tokens = self
                .parse(
                    prefix.clone(),
                    format!("could not parse prefix for struct `{}`", name),
                )
                .into_token_stream();
            tokens.extend(ignore_whitespace.clone());
            tokens
        });
        let suffix = attrs.suffix.clone().map_or(Default::default(), |suffix| {
            where_clause_types.push(suffix.clone());

            let mut tokens = self
                .parse(
                    suffix.clone(),
                    format!("could not parse suffix for struct `{}`", name),
                )
                .into_token_stream();
            tokens.extend(ignore_whitespace.clone());
            tokens
        });

        let parse_where_clause = where_clause_types
            .iter()
            .map(|ty| quote! {#ty: ::nommy::Parse<#parse_type>});

        tokens.extend(quote! {
            #[automatically_derived]
            impl #impl_args ::nommy::Parse<#parse_type> for #name #args
            where #(
                #parse_where_clause,
            )* {
                fn parse(input: &mut ::nommy::Buffer<impl ::std::iter::Iterator<Item=#parse_type>>) -> ::nommy::eyre::Result<Self> {
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

struct Peek {
    pub peeker: syn::Type,
    pub peek_type: syn::Ident,
}

impl ToTokens for Peek {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Peek { peeker, peek_type } = self;
        tokens.extend(quote! {
            if !<#peeker as ::nommy::Peek<#peek_type>>::peek(input) { return false }
        })
    }
}

struct Parse {
    pub parser: syn::Type,
    pub parse_type: syn::Ident,
    pub error: String,
    pub process: bool,
}

impl Parse {
    fn with_process(mut self, process: bool) -> Self {
        self.process = process;
        self
    }
}

impl ToTokens for Parse {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Parse {
            parser,
            parse_type,
            error,
            process,
        } = self;

        let process = if *process {
            quote! {.process()}
        } else {
            quote! {}
        };
        tokens.extend(quote! {
            <#parser as ::nommy::Parse<#parse_type>>::parse(input).wrap_err(#error)?#process;
        })
    }
}
