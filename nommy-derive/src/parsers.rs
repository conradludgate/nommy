use std::marker::PhantomData;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::attr::{FieldAttr};

#[derive(Debug, Clone)]
pub struct NamedField {
    pub attrs: FieldAttr,
    pub name: syn::Ident,
    pub ty: syn::Type,
}

impl From<syn::Field> for NamedField {
    fn from(field: syn::Field) -> Self {
        let syn::Field {
            ident, attrs, ty, ..
        } = field;
        let attrs = FieldAttr::parse_attrs(attrs);
        NamedField {
            attrs,
            name: ident.unwrap(),
            ty,
        }
    }
}

impl From<syn::Field> for UnnamedField {
    fn from(field: syn::Field) -> Self {
        let syn::Field {
            attrs, ty, ..
        } = field;
        let attrs = FieldAttr::parse_attrs(attrs);
        UnnamedField {
            attrs,
            ty,
        }
    }
}

#[derive(Debug, Clone)]
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

pub trait PTokens {
    const ASSIGN: bool;
    fn tokens(
        ty: &syn::Type,
        generic: &syn::Type,
        error: impl AsRef<str>,
        process: bool,
    ) -> TokenStream;
}

pub struct Parser;
impl PTokens for Parser {
    const ASSIGN: bool = true;
    fn tokens(
        ty: &syn::Type,
        generic: &syn::Type,
        error: impl AsRef<str>,
        process: bool,
    ) -> TokenStream {
        let error = error.as_ref();
        if process {
            quote! {
                <#ty as ::nommy::Process>::process(
                    <#ty as ::nommy::Parse<#generic>>::parse(input).wrap_err(#error)?
                );
            }
        } else {
            quote! {
                <#ty as ::nommy::Parse<#generic>>::parse(input).wrap_err(#error)?;
            }
        }
    }
}

pub struct Peeker;
impl PTokens for Peeker {
    const ASSIGN: bool = false;
    fn tokens(ty: &syn::Type, generic: &syn::Type, _: impl AsRef<str>, _: bool) -> TokenStream {
        quote! {
            if !(<#ty as ::nommy::Peek<#generic>>::peek(input)) { return false }
        }
    }
}

pub struct FunctionBuilder<'a, P: PTokens> {
    pub wc: &'a mut Vec<syn::Type>,
    pub generic: &'a syn::Type,
    after_each: TokenStream,
    _phantom: PhantomData<P>,
}

impl<'a, P: PTokens> FunctionBuilder<'a, P> {
    pub fn new(
        wc: &'a mut Vec<syn::Type>,
        generic: &'a syn::Type,
        ignore: &Vec<syn::Type>,
    ) -> Self {
        let mut after_each = TokenStream::new();
        for ty in ignore {
            wc.push(ty.clone());
            after_each.extend(P::tokens(
                &ty,
                &generic,
                "parsing whitespace should not fail, but did",
                false,
            ))
        }

        FunctionBuilder {
            wc,
            generic,
            after_each,
            _phantom: PhantomData,
        }
    }

    // prefix or suffix
    pub fn fix(
        &mut self,
        fix: &Option<syn::Type>,
        fix_type: &'static str,
        type_name: impl AsRef<str>,
    ) -> TokenStream {
        match fix {
            Some(prefix) => {
                self.wc.push(prefix.clone());
                let mut tokens = P::tokens(
                    &prefix,
                    &self.generic,
                    format!("failed to parse {} for {}", fix_type, type_name.as_ref()),
                    false,
                );
                tokens.extend(self.after_each.clone());
                tokens
            }
            None => Default::default(),
        }
    }

    // prefix or suffix
    pub fn field<F: FieldType>(&mut self, field: &F, field_num: usize) -> TokenStream {
        let ty = field.ty();
        let name = field.name(field_num);
        let attrs = field.attrs();

        let mut tokens = TokenStream::new();

        tokens.extend(self.fix(&attrs.prefix, "prefix", format!("field `{}`", name)));

        if P::ASSIGN {
            tokens.extend(quote! { let #name = });
        }
        let (parser, process) = match &attrs.parser {
            Some(p) => (p, true),
            None => (ty, false),
        };
        self.wc.push(parser.clone());
        tokens.extend(P::tokens(
            &parser,
            &self.generic,
            format!("failed to parse field `{}`", name),
            process,
        ));
        tokens.extend(self.after_each.clone());

        tokens.extend(self.fix(&attrs.suffix, "suffix", format!("field `{}`", name)));

        tokens
    }
}
