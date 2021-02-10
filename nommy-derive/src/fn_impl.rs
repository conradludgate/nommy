use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    attr::GlobalAttr,
    parsers::{
        ignore_impl, parser_parse_tokens, parser_parse_vec_tokens, parser_peek_tokens,
        peeker_peek_tokens, peeker_peek_vec_tokens, where_tokens, FieldType,
    },
};

#[derive(Default)]
pub struct BuildOutput {
    pub peek_impl: TokenStream,
    pub parse_impl: TokenStream,
    pub wc: TokenStream,
}

pub struct Builder<'a> {
    generic: &'a syn::Type,
    type_name: &'a syn::Ident,

    peek_impl: TokenStream,
    parse_impl: TokenStream,
    wc: TokenStream,
    after_each: TokenStream,
}

impl<'a> Builder<'a> {
    pub fn new(generic: &'a syn::Type, type_name: &'a syn::Ident) -> Self {
        Self {
            generic,
            type_name,
            peek_impl: TokenStream::new(),
            parse_impl: TokenStream::new(),
            wc: TokenStream::new(),
            after_each: TokenStream::new(),
        }
    }

    pub fn build(self) -> BuildOutput {
        let Builder {
            peek_impl,
            parse_impl,
            wc,
            ..
        } = self;
        BuildOutput {
            peek_impl,
            parse_impl,
            wc,
        }
    }
    pub fn create_ignore(&mut self, ignore: &[syn::Type]) {
        let (ignore_impl, after_each) =
            ignore_impl(&mut self.wc, ignore, self.generic, self.type_name);
        self.after_each = after_each;
        self.peek_impl.extend(ignore_impl.clone());
        self.parse_impl.extend(ignore_impl);
    }
    pub fn ignore(&mut self) {
        self.peek_impl.extend(self.after_each.clone());
        self.parse_impl.extend(self.after_each.clone());
    }
    pub fn add_fix(&mut self, fix: &Option<syn::Type>, fix_type: &'static str, name: String) {
        if let Some(fix) = fix {
            self.add_where(&fix);
            self.parse_impl.extend(parser_peek_tokens(
                &fix,
                &self.generic,
                &format!("failed to parse {} for {}", fix_type, name),
            ));
            self.peek_impl
                .extend(peeker_peek_tokens(&fix, &self.generic));
            self.ignore();
        }
    }
    pub fn add_field<F: FieldType>(&mut self, field: &F, field_num: usize) {
        let ty = field.ty();
        let name = field.name(field_num);
        let attrs = field.attrs();

        self.add_fix(&attrs.prefix, "prefix", format!("field `{}`", name));

        if attrs.vec.is_some() {
            let parser: Option<&syn::Type> = (&attrs.vec.parser).into();
            let parser = parser.unwrap();
            self.add_where(&parser);
            self.parse_impl.extend(parser_parse_vec_tokens(
                &name,
                &attrs.vec,
                &self.generic,
                &self.after_each,
            ));
            self.peek_impl.extend(peeker_peek_vec_tokens(
                &parser,
                &self.generic,
                &self.after_each,
            ));
        } else {
            let parser: Option<&syn::Type> = (&attrs.parser).into();
            let parser = parser.unwrap_or(&ty);
            self.add_where(&parser);
            self.parse_impl.extend(parser_parse_tokens(
                &name,
                &parser,
                &self.generic,
                &format!("failed to parse field `{}`", name),
            ));
            self.peek_impl
                .extend(peeker_peek_tokens(&parser, &self.generic));
            self.ignore();
        }

        self.add_fix(&attrs.suffix, "suffix", format!("field `{}`", name));
    }
    pub fn add_where(&mut self, ty: &syn::Type) {
        self.wc
            .extend(where_tokens(self.type_name, ty, self.generic));
    }
    pub fn add_where_raw(&mut self, tokens: TokenStream) {
        self.wc.extend(tokens);
    }

    pub fn start_variants(&mut self) {
        self.parse_impl.extend(quote! { let result = });
        self.peek_impl.extend(quote! { if });
    }
    pub fn add_variant(&mut self, peek_name: &syn::Ident, parse_name: &syn::Ident) {
        self.parse_impl.extend(quote! {
            if Self::#peek_name(&mut input.cursor()) {
                Self::#parse_name(input).wrap_err("variant peek succeeded but parse failed")
            } else
        });
        self.peek_impl.extend(quote! {
            !Self::#peek_name(&mut input.cursor()) &&
        });
    }
    pub fn finish_variants(&mut self, error: String) {
        self.parse_impl.extend(quote! {
            { Err(::nommy::eyre::eyre!(#error)) }?;
        });
        self.peek_impl.extend(quote! {
            true { return false; }
        });
    }
}

pub trait FnImpl<F: FieldType>: Sized {
    const TYPE: &'static str;
    fn name(&self) -> &syn::Ident;
    fn fields(&self) -> &[F];
    fn attrs(&self) -> &GlobalAttr;
    fn generic(&self) -> &syn::Type;

    fn build(&self) -> BuildOutput {
        let mut builder = Builder::new(self.generic(), self.name());
        let attrs = self.attrs();

        builder.create_ignore(&attrs.ignore);
        builder.add_fix(
            &attrs.prefix,
            "prefix",
            format!("{} `{}`", Self::TYPE, self.name()),
        );

        let fields = self.fields();
        for (field_num, field) in fields.iter().enumerate() {
            builder.add_field(field, field_num)
        }

        builder.add_fix(
            &attrs.suffix,
            "suffix",
            format!("{} `{}`", Self::TYPE, self.name()),
        );

        builder.build()
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
