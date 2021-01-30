use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::attr::{FieldAttr, GlobalAttr, IgnoreWS};

#[derive(Debug, Clone)]
pub struct NamedField {
    pub attrs: FieldAttr,
    pub name: syn::Ident,
    pub ty: syn::Type,
}

#[derive(Debug, Clone)]
pub struct UnnamedField {
    pub attrs: FieldAttr,
    pub ty: syn::Type,
}

struct Peeker<'a> {
    pub peek_type: &'a syn::Ident,
}

impl<'a> Peeker<'a> {
    pub fn peek(
        &self,
        wc: &'a mut Vec<syn::Type>,
        tokens: &mut TokenStream,
        after_each: TokenStream,
        peeker: &syn::Type,
    ) {
        wc.push(peeker.clone());
        tokens.extend(self.peek_tokens(&peeker));
        tokens.extend(after_each.clone());
    }

    fn peek_tokens(&self, peeker: &syn::Type) -> TokenStream {
        let peek_type = &self.peek_type;
        quote! {
            if !(<#peeker as ::nommy::Peek<#peek_type>>::peek(input)) { return false }
        }
    }
}

pub struct FieldPeeker<'a> {
    pub attrs: &'a GlobalAttr,
    pub peek_type: &'a syn::Ident,
    pub fields: Vec<UnnamedField>,
}

impl<'a> FieldPeeker<'a> {
    pub fn to_tokens(&self, wc: &'a mut Vec<syn::Type>) -> TokenStream {
        let peek_tokens = Peeker {
            peek_type: self.peek_type,
        };

        let mut after_each = TokenStream::new();
        if let Some(ws) = &self.attrs.ignore_whitespace {
            let ty: syn::Type = match ws {
                IgnoreWS::Spaces => {
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::Space>}).unwrap()
                }
                IgnoreWS::All => {
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::WhiteSpace>}).unwrap()
                }
            };
            peek_tokens.peek(wc, &mut after_each, Default::default(), &ty);
        }

        let mut tokens = TokenStream::new();

        if let Some(prefix) = self.attrs.prefix.clone() {
            peek_tokens.peek(wc, &mut tokens, after_each.clone(), &prefix);
        }

        for field in &self.fields {
            let UnnamedField { attrs, ty } = field;

            if let Some(prefix) = &attrs.prefix {
                peek_tokens.peek(wc, &mut tokens, after_each.clone(), &prefix);
            }

            let parser = match &attrs.parser {
                Some(p) => p,
                None => ty,
            };
            peek_tokens.peek(wc, &mut tokens, after_each.clone(), &parser);

            if let Some(suffix) = &attrs.suffix {
                peek_tokens.peek(wc, &mut tokens, after_each.clone(), &suffix);
            }
        }

        if let Some(suffix) = self.attrs.suffix.clone() {
            peek_tokens.peek(wc, &mut tokens, after_each.clone(), &suffix);
        }

        tokens
    }
}

struct Parser<'a> {
    pub parse_type: &'a syn::Ident,
}

impl<'a> Parser<'a> {
    pub fn parse(
        &self,
        wc: &'a mut Vec<syn::Type>,
        tokens: &mut TokenStream,
        after_each: TokenStream,
        parser: syn::Type,
        error: String,
        process: bool,
    ) {
        wc.push(parser.clone());
        tokens.extend(self.parse_tokens(&parser, error, process));
        tokens.extend(after_each.clone());
    }

    fn parse_tokens(&self, parser: &syn::Type, error: String, process: bool) -> TokenStream {
        let parse_type = &self.parse_type;
        if process {
            quote! {
                <#parser as ::nommy::Process>::process(
                    <#parser as ::nommy::Parse<#parse_type>>::parse(input).wrap_err(#error)?
                );
            }
        } else {
            quote! {
                <#parser as ::nommy::Parse<#parse_type>>::parse(input).wrap_err(#error)?;
            }
        }
    }
}

pub struct NamedFieldParser<'a> {
    pub struct_path: syn::Path,
    pub attrs: &'a GlobalAttr,
    pub parse_type: &'a syn::Ident,
    pub fields: Vec<NamedField>,
}

impl<'a> NamedFieldParser<'a> {
    pub fn to_tokens(&self, wc: &'a mut Vec<syn::Type>) -> TokenStream {
        let parse_tokens = Parser {
            parse_type: self.parse_type,
        };

        let mut tokens = TokenStream::new();

        let mut after_each = TokenStream::new();
        if let Some(ws) = &self.attrs.ignore_whitespace {
            let ty: syn::Type = match ws {
                IgnoreWS::Spaces => {
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::Space>}).unwrap()
                }
                IgnoreWS::All => {
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::WhiteSpace>}).unwrap()
                }
            };
            parse_tokens.parse(
                wc,
                &mut after_each,
                Default::default(),
                ty,
                "parsing whitespace should not fail, but did".to_string(),
                false,
            );
        }

        let struct_path = self.struct_path.clone().into_token_stream();

        if let Some(prefix) = self.attrs.prefix.clone() {
            parse_tokens.parse(
                wc,
                &mut tokens,
                after_each.clone(),
                prefix,
                format!("failed to parse prefix for struct `{}`", struct_path),
                false,
            );
        }

        let mut output = TokenStream::new();

        for field in &self.fields {
            let NamedField { attrs, name, ty } = field;

            if let Some(prefix) = attrs.prefix.clone() {
                parse_tokens.parse(
                    wc,
                    &mut tokens,
                    after_each.clone(),
                    prefix,
                    format!("failed to parse prefix for field `{}`", name),
                    false,
                );
            }

            let parser = match &attrs.parser {
                Some(p) => p,
                None => ty,
            };
            tokens.extend(quote! {let #name = });
            parse_tokens.parse(
                wc,
                &mut tokens,
                after_each.clone(),
                parser.clone(),
                format!("could not parse field `{}`", name),
                attrs.parser.is_some(),
            );

            if let Some(suffix) = attrs.suffix.clone() {
                parse_tokens.parse(
                    wc,
                    &mut tokens,
                    after_each.clone(),
                    suffix,
                    format!("failed to parse suffix for field `{}`", name),
                    false,
                );
            }

            output.extend(quote! { #name: #name.into(), })
        }

        if let Some(suffix) = self.attrs.suffix.clone() {
            parse_tokens.parse(
                wc,
                &mut tokens,
                after_each.clone(),
                suffix,
                format!("failed to parse suffix for struct `{}`", struct_path),
                false,
            );
        }

        tokens.extend(quote! {
            Ok(#struct_path{#output})
        });

        tokens
    }
}

pub struct UnnamedFieldParser<'a> {
    pub tuple_path: syn::Path,
    pub attrs: &'a GlobalAttr,
    pub parse_type: &'a syn::Ident,
    pub fields: Vec<UnnamedField>,
}

impl<'a> UnnamedFieldParser<'a> {
    pub fn to_tokens(&self, wc: &'a mut Vec<syn::Type>) -> TokenStream {
        let parse_tokens = Parser {
            parse_type: self.parse_type,
        };

        let mut tokens = TokenStream::new();

        let mut after_each = TokenStream::new();
        if let Some(ws) = &self.attrs.ignore_whitespace {
            let ty: syn::Type = match ws {
                IgnoreWS::Spaces => {
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::Space>}).unwrap()
                }
                IgnoreWS::All => {
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::WhiteSpace>}).unwrap()
                }
            };
            parse_tokens.parse(
                wc,
                &mut after_each,
                Default::default(),
                ty,
                "parsing whitespace should not fail, but did".to_string(),
                false,
            );
        }

        let tuple_path = self.tuple_path.clone().into_token_stream();

        if let Some(prefix) = self.attrs.prefix.clone() {
            parse_tokens.parse(
                wc,
                &mut tokens,
                after_each.clone(),
                prefix,
                format!("failed to parse prefix for tuple `{}`", tuple_path),
                false,
            );
        }

        let mut output = TokenStream::new();

        for (i, field) in self.fields.iter().enumerate() {
            let UnnamedField { attrs, ty } = field;
            let name = format_ident!("field{}", i);

            if let Some(prefix) = attrs.prefix.clone() {
                parse_tokens.parse(
                    wc,
                    &mut tokens,
                    after_each.clone(),
                    prefix,
                    format!("failed to parse prefix for field `{}`", name),
                    false,
                );
            }

            let parser = match &attrs.parser {
                Some(p) => p,
                None => ty,
            };
            tokens.extend(quote! {let #name = });
            parse_tokens.parse(
                wc,
                &mut tokens,
                after_each.clone(),
                parser.clone(),
                format!("could not parse field `{}`", name),
                attrs.parser.is_some(),
            );

            if let Some(suffix) = attrs.suffix.clone() {
                parse_tokens.parse(
                    wc,
                    &mut tokens,
                    after_each.clone(),
                    suffix,
                    format!("failed to parse tuple for field `{}`", name),
                    false,
                );
            }

            output.extend(quote! { #name.into(), })
        }

        if let Some(suffix) = self.attrs.suffix.clone() {
            parse_tokens.parse(
                wc,
                &mut tokens,
                after_each.clone(),
                suffix,
                format!("failed to parse suffix for tuple `{}`", tuple_path),
                false,
            );
        }

        tokens.extend(quote! {
            Ok(#tuple_path(#output))
        });

        tokens
    }
}

pub fn path_from_ident(ident: syn::Ident) -> syn::Path {
    let mut segments = syn::punctuated::Punctuated::new();
    segments.push(syn::PathSegment {
        ident,
        arguments: Default::default(),
    });

    syn::Path {
        leading_colon: None,
        segments: segments,
    }
}

pub fn path_from_idents(idents: Vec<syn::Ident>) -> syn::Path {
    let mut segments = syn::punctuated::Punctuated::new();
    for ident in idents {
        segments.push(syn::PathSegment {
            ident,
            arguments: Default::default(),
        });
    }

    syn::Path {
        leading_colon: None,
        segments: segments,
    }
}
