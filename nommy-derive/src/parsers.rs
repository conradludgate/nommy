use std::convert::TryFrom;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

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
    pub type_name: &'a syn::Ident,
    after_each: TokenStream,
}

impl<'a> FunctionBuilder<'a> {
    pub fn new(
        wc: &'a mut TokenStream,
        generic: &'a syn::Type,
        type_name: &'a syn::Ident,
        after_each: TokenStream,
    ) -> Self {
        FunctionBuilder {
            wc,
            generic,
            type_name,
            after_each,
        }
    }

    fn add_where(&mut self, ty: &syn::Type) {
        self.wc
            .extend(where_tokens(&self.type_name, &ty, &self.generic));
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
                self.add_where(&fix);
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
                self.add_where(&fix);
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
            self.add_where(&parser);
            tokens.extend(parser_parse_vec_tokens(
                &name,
                &attrs.vec,
                &self.generic,
                &self.after_each,
            ));
        } else {
            let parser: Option<&syn::Type> = (&attrs.parser).into();
            let parser = parser.unwrap_or(&ty);
            self.add_where(&parser);
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
            self.add_where(&parser);
            tokens.extend(peeker_peek_vec_tokens(
                &parser,
                &self.generic,
                &self.after_each,
            ));
        } else {
            let parser: Option<&syn::Type> = (&attrs.parser).into();
            let parser = parser.unwrap_or(&ty);
            self.add_where(&parser);
            tokens.extend(peeker_peek_tokens(&parser, &self.generic));
        }

        tokens.extend(self.after_each.clone());

        tokens.extend(self.peek_fix(&attrs.suffix));

        tokens
    }
}

fn where_tokens(type_name: &syn::Ident, ty: &syn::Type, generic: &syn::Type) -> TokenStream {
    if crate::ty::contains(&ty, &type_name) {
        quote! {}
    } else {
        quote! {#ty: ::nommy::Parse<#generic>,}
    }
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
        if !(<#ty as ::nommy::Parse<#generic>>::peek(input)) { return Err(::nommy::eyre::eyre!(#error)) }
    }
}
/// section of a parser impl
fn parser_parse_vec_tokens(
    name: &syn::Ident,
    attrs: &VecFieldAttr,
    generic: &syn::Type,
    after_each: &TokenStream,
) -> TokenStream {
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
            let pos = cursor.position();
            input.fast_forward(pos);

            #after_each
        };
    }
}
/// section of a peeker impl
fn peeker_peek_tokens(ty: &syn::Type, generic: &syn::Type) -> TokenStream {
    quote! {
        if !(<#ty as ::nommy::Parse<#generic>>::peek(input)) { return false }
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
fn peeker_peek_vec_tokens(
    ty: &syn::Type,
    generic: &syn::Type,
    after_each: &TokenStream,
) -> TokenStream {
    quote! {
        loop {
            let mut cursor = input.cursor();
            if !<#ty as ::nommy::Parse<#generic>>::peek(&mut cursor) {
                break;
            }
            let pos = cursor.position();
            input.fast_forward(pos);

            #after_each
        }
    }
}

pub fn ignore_impl(
    wc: &mut TokenStream,
    ignore: &[syn::Type],
    generic: &syn::Type,
    type_name: &syn::Ident,
) -> (TokenStream, TokenStream) {
    if ignore.is_empty() {
        return (TokenStream::new(), TokenStream::new());
    }

    let mut ignore_impl = TokenStream::new();
    let mut ignore_wc = TokenStream::new();
    for ty in ignore {
        let ty_string = ty.to_token_stream().to_string();
        wc.extend(where_tokens(&type_name, &ty, &generic));
        ignore_wc.extend(where_tokens(&type_name, &ty, &generic));
        ignore_impl.extend(quote! {
            {
                let mut cursor = input.cursor();
                if <#ty as ::nommy::Parse<#generic>>::peek(&mut cursor) {
                    let pos = cursor.position();
                    if ::std::cfg!(debug_assertions) && pos == 0 {
                        panic!(format!("ignore type `{}` passed but read 0 elements. Please ensure it reads at least 1 element otherwise it will cause an infinite loop", #ty_string));
                    }
                    input.fast_forward(pos);
                    return true
                }
            }
        });
    }
    let ignore_impl = quote! {
        struct __ParseIgnore;
        impl<#generic> ::nommy::Parse<#generic> for __ParseIgnore where #ignore_wc {
            fn parse(_: &mut impl ::nommy::Buffer<#generic>) -> ::nommy::eyre::Result<Self> {
                unimplemented!()
            }
            fn peek(input: &mut impl ::nommy::Buffer<#generic>) -> bool {
                #ignore_impl

                false
            }
        }
    };

    let after_each = quote! {
        <::std::vec::Vec<__ParseIgnore> as ::nommy::Parse<#generic>>::peek(input);
    };

    (ignore_impl, after_each)
}
