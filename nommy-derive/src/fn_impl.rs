use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    attr::GlobalAttr,
    parsers::{FieldType, FunctionBuilder, PTokens, Parser, Peeker},
};

pub struct BuildOutput {
    pub fn_impl: TokenStream,
    pub wc: Vec<syn::Type>,
}

fn build<T: FieldType, F: FnImpl<T>, P: PTokens>(fn_impl: &F) -> BuildOutput {
    let mut wc = vec![];
    let name = fn_impl.name();
    let attrs = fn_impl.attrs();

    let mut builder =
        FunctionBuilder::<P>::new(&mut wc, fn_impl.generic(), &attrs.ignore_whitespace);

    let mut tokens = TokenStream::new();

    tokens.extend(builder.fix(&attrs.prefix, "prefix", format!("{} `{}`", F::TYPE, name)));

    let fields = fn_impl.fields();
    for (i, field) in fields.iter().enumerate() {
        tokens.extend(builder.field(field, i))
    }

    tokens.extend(builder.fix(&attrs.suffix, "suffix", format!("{} `{}`", F::TYPE, name)));

    BuildOutput {
        fn_impl: tokens,
        wc,
    }
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
        let BuildOutput { mut fn_impl, wc } = build::<F, Self, Parser>(&self);
        fn_impl.extend(self.result());
        BuildOutput { fn_impl, wc }
    }

    fn build_peek(&self) -> BuildOutput {
        build::<F, Self, Peeker>(&self)
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
                    segments: segments,
                },
            })
        }
    }

}
