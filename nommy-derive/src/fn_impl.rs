use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{attr::GlobalAttr, parsers::{FieldType, FunctionBuilder, ignore_impl}};

pub struct BuildOutput {
    pub fn_impl: TokenStream,
    pub wc: TokenStream,
}

pub trait FnImpl<F: FieldType>: Sized {
    const TYPE: &'static str;
    fn name(&self) -> &syn::Ident;
    fn fields(&self) -> &[F];
    fn attrs(&self) -> &GlobalAttr;
    fn generic(&self) -> &syn::Type;
    fn after_each(&self) -> TokenStream {
        quote! {}
    }
    fn result(&self) -> TokenStream;

    fn build_parse(&self) -> BuildOutput {
        let mut wc = TokenStream::new();
        let name = self.name();
        let attrs = self.attrs();

        let (ignore, after_each) = ignore_impl(&mut wc, &attrs.ignore, self.generic());

        let mut builder = FunctionBuilder::new(&mut wc, self.generic(), after_each);

        let mut tokens = TokenStream::new();
        tokens.extend(ignore);

        tokens.extend(builder.parse_fix(
            &attrs.prefix,
            "prefix",
            &format!("{} `{}`", Self::TYPE, name),
        ));

        let fields = self.fields();
        for (i, field) in fields.iter().enumerate() {
            tokens.extend(builder.parse_field(field, i))
        }

        tokens.extend(builder.parse_fix(
            &attrs.suffix,
            "suffix",
            &format!("{} `{}`", Self::TYPE, name),
        ));

        tokens.extend(self.result());
        BuildOutput {
            fn_impl: tokens,
            wc,
        }
    }

    fn build_peek(&self) -> BuildOutput {
        let mut wc = TokenStream::new();
        let attrs = self.attrs();

        let (ignore, after_each) = ignore_impl(&mut wc, &attrs.ignore, self.generic());
        let mut builder = FunctionBuilder::new(&mut wc, self.generic(), after_each);

        let mut tokens = TokenStream::new();
        tokens.extend(ignore);

        tokens.extend(builder.peek_fix(&attrs.prefix));

        let fields = self.fields();
        for (i, field) in fields.iter().enumerate() {
            tokens.extend(builder.peek_field(field, i))
        }

        tokens.extend(builder.peek_fix(&attrs.suffix));
        BuildOutput {
            fn_impl: tokens,
            wc,
        }
    }
}

pub fn parse_or(parse_type: &Option<syn::Type>) -> syn::Type {
    match &parse_type {
        Some(pt) => pt.clone(),
        None => {
            let mut segments = syn::punctuated::Punctuated::new();
            segments.push(syn::PathSegment {
                ident: format_ident!("__ParseGenericType"),
                arguments: Default::default(),
            });

            syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments,
                },
            })
        }
    }
}
