use std::convert::TryFrom;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::attr::{FieldAttr, VecFieldAttr};

pub struct NamedField {
    pub attrs: FieldAttr,
    pub name: syn::Ident,
    pub ty: syn::Type,
}

impl TryFrom<syn::Field> for NamedField {
    type Error = syn::Error;
    fn try_from(field: syn::Field) -> syn::Result<Self> {
        let syn::Field {
            ident, attrs, ty, ..
        } = field;
        let attrs = FieldAttr::parse_attrs(attrs)?;
        Ok(NamedField {
            attrs,
            name: ident.unwrap(),
            ty,
        })
    }
}

impl TryFrom<syn::Field> for UnnamedField {
    type Error = syn::Error;
    fn try_from(field: syn::Field) -> syn::Result<Self> {
        let syn::Field { attrs, ty, .. } = field;
        let attrs = FieldAttr::parse_attrs(attrs)?;
        Ok(UnnamedField { attrs, ty })
    }
}

pub struct UnnamedField {
    pub attrs: FieldAttr,
    pub ty: syn::Type,
}

pub trait FieldType {
    fn ty(&self) -> &syn::Type;
    fn name(&self, i: usize) -> syn::Ident;
    fn attrs(&self) -> &FieldAttr;
}

impl FieldType for NamedField {
    fn ty(&self) -> &syn::Type {
        &self.ty
    }
    fn name(&self, _: usize) -> syn::Ident {
        self.name.clone()
    }
    fn attrs(&self) -> &FieldAttr {
        &self.attrs
    }
}

impl FieldType for UnnamedField {
    fn ty(&self) -> &syn::Type {
        &self.ty
    }
    fn name(&self, i: usize) -> syn::Ident {
        format_ident!("elem{}", i)
    }
    fn attrs(&self) -> &FieldAttr {
        &self.attrs
    }
}

pub struct FunctionBuilder<'a> {
    pub wc: &'a mut TokenStream,
    pub generic: &'a syn::Type,
    after_each: TokenStream,
}

impl<'a> FunctionBuilder<'a> {
    pub fn new(
        wc: &'a mut TokenStream,
        generic: &'a syn::Type,
        after_each: TokenStream,
    ) -> Self {
        FunctionBuilder {
            wc,
            generic,
            after_each,
        }
    }

    fn peek_where(&mut self, ty: &syn::Type) {
        self.wc.extend(peek_where_tokens(&ty, &self.generic));
    }

    fn parse_where(&mut self, ty: &syn::Type) {
        self.wc.extend(parse_where_tokens(&ty, &self.generic));
    }

    // prefix or suffix
    pub fn parse_fix(
        &mut self,
        fix: &Option<syn::Type>,
        fix_type: &'static str,
        type_name: &str,
    ) -> TokenStream {
        match fix {
            Some(fix) => {
                self.peek_where(&fix);
                let mut tokens = parser_peek_tokens(
                    &fix,
                    &self.generic,
                    &format!("failed to parse {} for {}", fix_type, type_name),
                );
                tokens.extend(self.after_each.clone());
                tokens
            }
            None => Default::default(),
        }
    }

    // prefix or suffix
    pub fn peek_fix(&mut self, fix: &Option<syn::Type>) -> TokenStream {
        match fix {
            Some(fix) => {
                self.peek_where(&fix);
                let mut tokens = peeker_peek_tokens(&fix, &self.generic);
                tokens.extend(self.after_each.clone());
                tokens
            }
            None => Default::default(),
        }
    }

    pub fn parse_field<F: FieldType>(&mut self, field: &F, field_num: usize) -> TokenStream {
        let ty = field.ty();
        let name = field.name(field_num);
        let attrs = field.attrs();

        let mut tokens = TokenStream::new();

        tokens.extend(self.parse_fix(&attrs.prefix, "prefix", &format!("field `{}`", name)));

        if attrs.vec.is_some() {
            let parser: Option<&syn::Type> = (&attrs.vec.parser).into();
            let parser = parser.unwrap();
            self.parse_where(&parser);
            tokens.extend(parser_parse_vec_tokens(&name, &attrs.vec, &self.generic));
        } else {
            let parser: Option<&syn::Type> = (&attrs.parser).into();
            let parser = parser.unwrap_or(&ty);
            self.parse_where(&parser);
            tokens.extend(parser_parse_tokens(
                &name,
                &parser,
                &self.generic,
                &format!("failed to parse field `{}`", name),
            ));
        }

        tokens.extend(self.after_each.clone());

        tokens.extend(self.parse_fix(&attrs.suffix, "suffix", &format!("field `{}`", name)));

        tokens
    }

    pub fn peek_field<F: FieldType>(&mut self, field: &F, _field_num: usize) -> TokenStream {
        let ty = field.ty();
        let attrs = field.attrs();

        let mut tokens = TokenStream::new();

        tokens.extend(self.peek_fix(&attrs.prefix));

        if attrs.vec.is_some() {
            let parser: Option<&syn::Type> = (&attrs.vec.parser).into();
            let parser = parser.unwrap();
            self.peek_where(&parser);
            tokens.extend(peeker_peek_vec_tokens(&parser, &self.generic));
        } else {
            let parser: Option<&syn::Type> = (&attrs.parser).into();
            let parser = parser.unwrap_or(&ty);
            self.peek_where(&parser);
            tokens.extend(peeker_peek_tokens(&parser, &self.generic));
        }

        tokens.extend(self.after_each.clone());

        tokens.extend(self.peek_fix(&attrs.suffix));

        tokens
    }
}

fn peek_where_tokens(ty: &syn::Type, generic: &syn::Type) -> TokenStream {
    quote! {#ty: ::nommy::Peek<#generic>,}
}
fn parse_where_tokens(ty: &syn::Type, generic: &syn::Type) -> TokenStream {
    quote! {#ty: ::nommy::Parse<#generic>,}
}
/// section of a parser impl
fn parser_parse_tokens(
    name: &syn::Ident,
    ty: &syn::Type,
    generic: &syn::Type,
    error: &str,
) -> TokenStream {
    quote! {
        let #name = <#ty as ::nommy::Parse<#generic>>::parse(input).wrap_err(#error)?.into();
    }
}
/// section of a parser impl
fn parser_peek_tokens(ty: &syn::Type, generic: &syn::Type, error: &str) -> TokenStream {
    quote! {
        if !(<#ty as ::nommy::Peek<#generic>>::peek(input)) { return Err(::nommy::eyre::eyre!(#error)) }
    }
}
/// section of a parser impl
fn parser_parse_vec_tokens(name: &syn::Ident, attrs: &VecFieldAttr, generic: &syn::Type) -> TokenStream {
    let parser: Option<&syn::Type> = (&attrs.parser).into();
    let parser = parser.unwrap();
    quote! {
        let mut #name = ::std::vec::Vec::new();
        loop {
            let mut cursor = input.cursor();
            match <#parser as ::nommy::Parse<#generic>>::parse(&mut cursor) {
                Ok(p) => #name.push(p.into()),
                _ => break,
            }
            cursor.fast_forward_parent();
        };
    }
}
/// section of a peeker impl
fn peeker_peek_tokens(ty: &syn::Type, generic: &syn::Type) -> TokenStream {
    quote! {
        if !(<#ty as ::nommy::Peek<#generic>>::peek(input)) { return false }
    }
}
/// section of a peeker impl
fn _peeker_parse_tokens(name: &syn::Ident, ty: &syn::Type, generic: &syn::Type) -> TokenStream {
    quote! {
        let #name = match <#ty as ::nommy::Parse<#generic>>::parse(input) {
            Ok(v) => v,
            _ => return false,
        };
    }
}
/// section of a peeker impl
fn peeker_peek_vec_tokens(ty: &syn::Type, generic: &syn::Type) -> TokenStream {
    quote! {
        loop {
            let mut cursor = input.cursor();
            if !<#ty as ::nommy::Peek<#generic>>::peek(&mut cursor) {
                break;
            }
            cursor.fast_forward_parent()
        }
    }
}

pub fn ignore_impl(
    wc: &mut TokenStream,
    ignore: &Vec<syn::Type>,
    generic: &syn::Type,
) -> (TokenStream, TokenStream) {
    if ignore.is_empty() {
        return (TokenStream::new(), TokenStream::new())
    }

    let mut ignore_impl = TokenStream::new();
    let mut ignore_wc = TokenStream::new();
    for ty in ignore {
        wc.extend(peek_where_tokens(&ty, &generic));
        ignore_wc.extend(peek_where_tokens(&ty, &generic));
        ignore_impl.extend(quote! {
            {
                let mut cursor = input.cursor();
                if <#ty as ::nommy::Peek<#generic>>::peek(&mut cursor) {
                    cursor.fast_forward_parent();
                    return true
                }
            }
        });
    }
    let ignore_impl = quote! {
        struct __ParseIgnore;
        impl<#generic> ::nommy::Peek<#generic> for __ParseIgnore where #ignore_wc {
            fn peek(input: &mut impl ::nommy::Buffer<#generic>) -> bool {
                #ignore_impl

                false
            }
        }
    };

    let after_each = quote! {
        <::std::vec::Vec<__ParseIgnore> as ::nommy::Peek<#generic>>::peek(input);
    };

    (ignore_impl, after_each)
}
