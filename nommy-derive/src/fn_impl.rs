use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
    attr::{GlobalAttr, VecFieldAttr},
    parsers::FieldType,
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
    parse_type: &'a Option<syn::Type>,

    peek_impl: TokenStream,
    parse_impl: TokenStream,
    wc: TokenStream,
    after_each: TokenStream,
}

impl<'a> Builder<'a> {
    pub fn new(
        generic: &'a syn::Type,
        type_name: &'a syn::Ident,
        parse_type: &'a Option<syn::Type>,
    ) -> Self {
        Self {
            generic,
            type_name,
            parse_type,
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
        let (ignore_impl, after_each) = self.ignore_impl(ignore);
        self.after_each = after_each;
        self.peek_impl.extend(ignore_impl.clone());
        self.parse_impl.extend(ignore_impl);
    }

    pub fn ignore(&mut self) {
        self.peek_impl.extend(self.after_each.clone());
        self.parse_impl.extend(self.after_each.clone());
    }

    pub fn add_where(&mut self, ty: &syn::Type) {
        self.wc.extend(self.where_tokens(ty));
    }
    pub fn add_where_raw(&mut self, tokens: TokenStream) {
        self.wc.extend(tokens);
    }

    pub fn add_fix(&mut self, fix: &Option<syn::Type>, fix_type: &'static str, name: String) {
        if let Some(fix) = fix {
            self.add_where(&fix);
            self.parse_impl.extend(
                self.parser_peek_tokens(
                    &fix,
                    &format!("failed to parse {} for {}", fix_type, name),
                ),
            );
            self.peek_impl.extend(self.peeker_peek_tokens(&fix));
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
            self.parse_impl
                .extend(self.parser_parse_vec_tokens(&name, &attrs.vec));
            self.peek_impl.extend(self.peeker_peek_vec_tokens(&parser));
        } else {
            let parser: Option<&syn::Type> = (&attrs.parser).into();
            let parser = parser.unwrap_or(&ty);
            self.add_where(&parser);
            self.parse_impl.extend(self.parser_parse_tokens(
                &name,
                &parser,
                &format!("failed to parse field `{}`", name),
            ));
            self.peek_impl.extend(self.peeker_peek_tokens(&parser));
            self.ignore();
        }

        self.add_fix(&attrs.suffix, "suffix", format!("field `{}`", name));
    }

    pub fn start_variants(&mut self) {
        self.parse_impl.extend(quote! {
            let mut cursor = input.cursor();
            let result =
        });
        self.peek_impl
            .extend(quote! { let mut cursor = input.cursor(); if });
    }
    pub fn add_variant(&mut self, peek_name: &syn::Ident, parse_name: &syn::Ident) {
        self.parse_impl.extend(quote! {
            if let (true, Ok(result)) = (cursor.reset_internal(), Self::#parse_name(&mut cursor)) {
                result
            } else
        });
        self.peek_impl.extend(quote! {
            !Self::#peek_name(&mut cursor) && cursor.reset_internal() &&
        });
    }
    pub fn finish_variants(&mut self, error: String) {
        self.parse_impl.extend(quote! {
            { return Err(::nommy::eyre::eyre!(#error)); };
            let pos = cursor.position();
            input.fast_forward(pos);
        });
        self.peek_impl.extend(quote! {
            true { return false; }
            let pos = cursor.position();
            input.fast_forward(pos);
        });
    }
}

pub struct FnImpl<'a, F> {
    pub ty: &'static str,
    pub name: &'a syn::Ident,
    pub fields: &'a [F],
    pub attrs: &'a GlobalAttr,
    pub generic: &'a syn::Type,
}

impl<'a, F: FieldType> FnImpl<'a, F> {
    pub fn build(&self, type_name: &syn::Ident) -> BuildOutput {
        let mut builder = Builder::new(self.generic, type_name, &self.attrs.parse_type);

        builder.create_ignore(&self.attrs.ignore);
        builder.add_fix(
            &self.attrs.prefix,
            "prefix",
            format!("{} `{}`", self.ty, self.name),
        );

        for (field_num, field) in self.fields.iter().enumerate() {
            builder.add_field(field, field_num)
        }

        builder.add_fix(
            &self.attrs.suffix,
            "suffix",
            format!("{} `{}`", self.ty, self.name),
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

impl<'a> Builder<'a> {
    fn where_tokens(&self, ty: &syn::Type) -> TokenStream {
        if crate::ty::contains(&ty, &self.type_name) {
            quote! {}
        } else {
            let generic = &self.generic;
            quote! {#ty: ::nommy::Parse<#generic>,}
        }
    }
    fn parser_peek_tokens(&self, ty: &syn::Type, error: &str) -> TokenStream {
        let generic = &self.generic;
        quote! {
            if !(<#ty as ::nommy::Parse<#generic>>::peek(input)) { return Err(::nommy::eyre::eyre!(#error)) }
        }
    }
    fn parser_parse_tokens(&self, name: &syn::Ident, ty: &syn::Type, error: &str) -> TokenStream {
        let generic = &self.generic;
        quote! {
            let #name = <#ty as ::nommy::Parse<#generic>>::parse(input).wrap_err(#error)?.try_into()?;
        }
    }
    fn peeker_peek_tokens(&self, ty: &syn::Type) -> TokenStream {
        let generic = &self.generic;
        quote! {
            if !(<#ty as ::nommy::Parse<#generic>>::peek(input)) { return false }
        }
    }

    fn parser_parse_vec_tokens(&self, name: &syn::Ident, attrs: &VecFieldAttr) -> TokenStream {
        let generic = &self.generic;

        let parser: Option<&syn::Type> = (&attrs.parser).into();
        let parser = parser.unwrap();

        let (min, max) = match &attrs.count {
            Some(count) => (quote! { #count }, quote! { #count }),
            None => {
                let min = match &attrs.min {
                    Some(min) => quote! { #min },
                    None => quote! { 0 },
                };
                let max = match &attrs.max {
                    Some(max) => quote! { #max },
                    None => quote! { usize::MAX },
                };
                (min, max)
            }
        };

        if let Some(sep) = &attrs.seperated_by {
            match &attrs.trailing {
                Some(true) => quote! {
                    let #name = ::nommy::vec::parse_vec_seperated_by_trailing::<#parser, _, #sep, __ParseIgnore, #generic, _>(#max, input)?;
                    if #name.len() < #min {
                        return Err(::nommy::eyre::eyre!("could not parse enough for vec"));
                    }
                },
                Some(false) => quote! {
                    let #name = ::nommy::vec::parse_vec_seperated_by_maybe_trailing::<#parser, _, #sep, __ParseIgnore, #generic, _>(#max, input)?;
                    if #name.len() < #min {
                        return Err(::nommy::eyre::eyre!("could not parse enough for vec"));
                    }
                },
                None => quote! {
                    let #name = ::nommy::vec::parse_vec_seperated_by::<#parser, _, #sep, __ParseIgnore, #generic, _>(#max, input)?;
                    if #name.len() < #min {
                        return Err(::nommy::eyre::eyre!("could not parse enough for vec"));
                    }
                },
            }
        } else {
            quote! {
                let #name = ::nommy::vec::parse_vec::<#parser, _, __ParseIgnore, #generic, _>(#max, input)?;
                if #name.len() < #min {
                    return Err(::nommy::eyre::eyre!("could not parse enough for vec"));
                }
            }
        }
    }

    pub fn peeker_peek_vec_tokens(&self, ty: &syn::Type) -> TokenStream {
        let generic = &self.generic;
        let after_each = &self.after_each;

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

    fn ignore_impl(&mut self, ignore: &[syn::Type]) -> (TokenStream, TokenStream) {
        let generic = &self.generic;

        let mut ignore_impl = TokenStream::new();
        let mut ignore_wc = TokenStream::new();
        for ty in ignore {
            let ty_string = ty.to_token_stream().to_string();
            self.wc.extend(self.where_tokens(&ty));
            ignore_wc.extend(self.where_tokens(&ty));
            ignore_impl.extend(quote! {
            {
                let mut cursor = input.cursor();
                if <#ty as ::nommy::Parse<#generic>>::peek(&mut cursor) {
                    let pos = cursor.position();
                    if ::std::cfg!(debug_assertions) && pos == 0 {
                        panic!("ignore type `{}` passed but read 0 elements. Please ensure it reads at least 1 element otherwise it will cause an infinite loop", #ty_string);
                    }
                    input.fast_forward(pos);
                    return true
                }
            }
        });
        }

        let impl_line = match self.parse_type {
            Some(_) => quote! {
                impl ::nommy::Parse<#generic> for __ParseIgnore
            },
            None => quote! {
                impl<#generic> ::nommy::Parse<#generic> for __ParseIgnore where #ignore_wc
            },
        };

        let ignore_impl = quote! {
            struct __ParseIgnore;
            #impl_line {
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
}
