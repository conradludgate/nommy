use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::attr::{FieldAttr, GlobalAttr, IgnoreWS};

use super::named_struct::NamedField;

#[derive(Clone)]
pub struct EnumInput {
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub fields: Vec<EnumField>,
}

impl EnumInput {
    pub fn new(
        name: syn::Ident,
        generics: syn::Generics,
        attrs: Vec<syn::Attribute>,
        enum_data: syn::DataEnum,
    ) -> Self {
        let args = generics.type_params().cloned().map(|tp| tp.ident).collect();

        let fields = enum_data
            .variants
            .iter()
            .map(|v| match &v.fields {
                syn::Fields::Named(named) => EnumField {
                    name: v.ident.clone(),
                    field_type: EnumFieldType::Named(
                        named
                            .named
                            .iter()
                            .cloned()
                            .map(|field| {
                                let syn::Field {
                                    ident, attrs, ty, ..
                                } = field;
                                let attrs = FieldAttr::parse_attrs(attrs);
                                NamedField {
                                    attrs,
                                    name: ident.unwrap(),
                                    ty,
                                }
                            })
                            .collect(),
                    ),
                },
                syn::Fields::Unnamed(tuple) => EnumField {
                    name: v.ident.clone(),
                    field_type: EnumFieldType::Tuple(
                        tuple
                            .unnamed
                            .iter()
                            .cloned()
                            .map(|field| {
                                let syn::Field { attrs, ty, .. } = field;
                                let attrs = FieldAttr::parse_attrs(attrs);
                                UnnamedField { attrs, ty }
                            })
                            .collect(),
                    ),
                },
                syn::Fields::Unit => panic!("Unit variants not supported in enum parse derive"),
            })
            .collect();

        let attrs = GlobalAttr::parse_attrs(attrs);

        EnumInput {
            name,
            attrs,
            args,
            fields,
        }
    }

    pub fn process(self) -> EnumOutput {
        EnumOutput {
            peek_impl: EnumPeek::new(self.clone()),
            parse_impl: EnumParse::new(self),
        }
    }
}

#[derive(Clone)]
pub struct EnumField {
    pub name: syn::Ident,
    pub field_type: EnumFieldType,
}

#[derive(Clone)]
pub enum EnumFieldType {
    // None, // not supported
    Tuple(Vec<UnnamedField>),
    Named(Vec<NamedField>),
}

#[derive(Debug, Clone)]
pub struct UnnamedField {
    pub attrs: FieldAttr,
    pub ty: syn::Type,
}

pub struct EnumOutput {
    peek_impl: EnumPeek,
    parse_impl: EnumParse,
}

impl ToTokens for EnumOutput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let EnumOutput {
            peek_impl,
            parse_impl,
        } = self;

        peek_impl.to_tokens(tokens);
        parse_impl.to_tokens(tokens);
    }
}

pub struct EnumPeek {
    pub fn_impl: TokenStream,
    pub peek_fn_names: Vec<syn::Ident>,
    pub peek_fn_impl: Vec<TokenStream>,
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub where_clause_types: Vec<syn::Type>,
    pub args: Vec<syn::Ident>,
    pub peek_type: syn::Ident,
    pub after_each: TokenStream,
}

impl EnumPeek {
    fn new(input: EnumInput) -> Self {
        let peek_type = format_ident!("__PeekType");

        let mut peek_impl = EnumPeek {
            fn_impl: Default::default(),
            peek_fn_names: vec![],
            peek_fn_impl: vec![],
            attrs: input.attrs,
            name: input.name,
            where_clause_types: vec![],
            args: input.args,
            peek_type,
            after_each: Default::default(),
        };

        peek_impl.enrich(input.fields);

        peek_impl
    }

    fn enrich(&mut self, fields: Vec<EnumField>) {
        if let Some(ws) = &self.attrs.ignore_whitespace {
            let ty: syn::Type = match ws {
                IgnoreWS::Spaces => {
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::Space>}).unwrap()
                }
                IgnoreWS::All => {
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::WhiteSpace>}).unwrap()
                }
            };
            self.where_clause_types.push(ty.clone());
            self.after_each.extend(self.peek_tokens(&ty))
        }

        let name = self.name.clone();
        self.fn_impl.extend(quote! {if true});
        for field in fields {
            let peek = format_ident!("__peek_{}", field.name.to_string().to_lowercase());
            self.fn_impl.extend(quote! {
                && !#name::#peek(&mut input.cursor())
            });

            self.peek_fn_names.push(peek);
            match field.field_type {
                EnumFieldType::Named(named) => self.add_struct_peek(named),
                EnumFieldType::Tuple(unnamed) => self.add_tuple_peek(unnamed),
            }
        }
        self.fn_impl.extend(quote! {{ return false; }});
    }

    fn peek(&mut self, peeker: syn::Type) -> TokenStream {
        self.where_clause_types.push(peeker.clone());
        let mut tokens = self.peek_tokens(&peeker);
        tokens.extend(self.after_each.clone());
        tokens
    }

    fn peek_tokens(&self, peeker: &syn::Type) -> TokenStream {
        let peek_type = &self.peek_type;
        quote! {
            if !(<#peeker as ::nommy::Peek<#peek_type>>::peek(input)) { return false }
        }
    }

    fn add_struct_peek(&mut self, named: Vec<NamedField>) {
        let mut tokens = TokenStream::new();

        if let Some(prefix) = self.attrs.prefix.clone() {
            tokens.extend(self.peek(prefix));
        }

        for field in named {
            let NamedField { attrs, name: _, ty } = field;

            if let Some(prefix) = attrs.prefix.clone() {
                tokens.extend(self.peek(prefix));
            }

            let parser = attrs.parser.clone().unwrap_or(ty.clone());
            tokens.extend(self.peek(parser));

            if let Some(suffix) = attrs.suffix.clone() {
                tokens.extend(self.peek(suffix));
            }
        }

        if let Some(suffix) = self.attrs.suffix.clone() {
            tokens.extend(self.peek(suffix));
        }

        self.peek_fn_impl.push(tokens)
    }

    fn add_tuple_peek(&mut self, unnamed: Vec<UnnamedField>) {
        let mut tokens = TokenStream::new();

        if let Some(prefix) = self.attrs.prefix.clone() {
            tokens.extend(self.peek(prefix));
        }

        for field in unnamed {
            let UnnamedField { attrs, ty } = field;

            if let Some(prefix) = attrs.prefix.clone() {
                tokens.extend(self.peek(prefix));
            }

            let parser = attrs.parser.clone().unwrap_or(ty.clone());
            tokens.extend(self.peek(parser));

            if let Some(suffix) = attrs.suffix.clone() {
                tokens.extend(self.peek(suffix));
            }
        }

        if let Some(suffix) = self.attrs.suffix.clone() {
            tokens.extend(self.peek(suffix));
        }

        self.peek_fn_impl.push(tokens)
    }
}

impl ToTokens for EnumPeek {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let EnumPeek {
            fn_impl,
            peek_fn_names,
            peek_fn_impl,
            attrs: _,
            after_each: _,
            name,
            where_clause_types,
            args,
            peek_type,
        } = self;

        let where_clause = quote!{
            where #(
                #where_clause_types: ::nommy::Peek<#peek_type>,
            )*
        };

        tokens.extend(quote!{
            #[automatically_derived]
            impl <#peek_type, #(#args),*> ::nommy::Peek<#peek_type> for #name<#(#args),*> #where_clause {
                fn peek(input: &mut ::nommy::Cursor<impl ::std::iter::Iterator<Item=#peek_type>>) -> bool {
                    #fn_impl
                    true
                }
            }

            #[automatically_derived]
            impl<#(#args),*> #name<#(#args),*> {
                #(
                    fn #peek_fn_names<#peek_type>(input: &mut ::nommy::Cursor<impl ::std::iter::Iterator<Item=#peek_type>>) -> bool #where_clause {
                        #peek_fn_impl
                        true
                    }
                )*
            }
        })
    }
}

pub struct EnumParse {
    pub fn_impl: TokenStream,
    pub parse_fn_names: Vec<syn::Ident>,
    pub parse_fn_impl: Vec<TokenStream>,
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub where_clause_types: Vec<syn::Type>,
    pub args: Vec<syn::Ident>,
    pub parse_type: syn::Ident,
    pub after_each: TokenStream,
}

impl EnumParse {
    fn new(input: EnumInput) -> Self {
        let parse_type = format_ident!("__ParseType");

        let mut parse_impl = EnumParse {
            fn_impl: Default::default(),
            parse_fn_names: vec![],
            parse_fn_impl: vec![],
            attrs: input.attrs,
            name: input.name,
            where_clause_types: vec![],
            args: input.args,
            parse_type,
            after_each: Default::default(),
        };

        parse_impl.enrich(input.fields);

        parse_impl
    }

    fn enrich(&mut self, fields: Vec<EnumField>) {
        if let Some(ws) = &self.attrs.ignore_whitespace {
            let ty: syn::Type = match ws {
                IgnoreWS::Spaces => {
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::Space>}).unwrap()
                }
                IgnoreWS::All => {
                    syn::parse2(quote! {::std::vec::Vec<::nommy::text::WhiteSpace>}).unwrap()
                }
            };
            self.where_clause_types.push(ty.clone());
            self.after_each.extend(self.parse_tokens(
                &ty,
                "parsing whitespace should not fail, but did".to_string(),
                false,
            ))
        }

        let name = self.name.clone();
        for field in fields {
            let peek = format_ident!("__peek_{}", field.name.to_string().to_lowercase());
            let parse = format_ident!("__parse_{}", field.name.to_string().to_lowercase());
            self.fn_impl.extend(quote! {
                if #name::#peek(&mut input.cursor()) {
                    #name::#parse(input)
                } else
            });

            self.parse_fn_names.push(parse);
            match field.field_type {
                EnumFieldType::Named(named) => self.add_struct_parse(field.name, named),
                EnumFieldType::Tuple(unnamed) => self.add_tuple_parse(field.name, unnamed),
            }
        }
    }

    fn parse(&mut self, parser: syn::Type, error: String, process: bool) -> TokenStream {
        self.where_clause_types.push(parser.clone());
        let mut tokens = self.parse_tokens(&parser, error, process);
        tokens.extend(self.after_each.clone());
        tokens
    }

    fn parse_tokens(&self, parser: &syn::Type, error: String, process: bool) -> TokenStream {
        let parse_type = &self.parse_type;
        if process {
            quote! {
                <#parser as ::nommy::Parse<#parse_type>>::parse(input).wrap_err(#error)?.process();
            }
        } else {
            quote! {
                <#parser as ::nommy::Parse<#parse_type>>::parse(input).wrap_err(#error)?;
            }
        }
    }

    fn add_struct_parse(&mut self, variant_name: syn::Ident, named: Vec<NamedField>) {
        let mut tokens = TokenStream::new();

        if let Some(prefix) = self.attrs.prefix.clone() {
            tokens.extend(self.parse(
                prefix,
                format!("failed to parse prefix for enum `{}`", self.name),
                false,
            ));
        }

        let mut output = TokenStream::new();

        for field in named {
            let NamedField { attrs, name, ty } = field;

            if let Some(prefix) = attrs.prefix.clone() {
                tokens.extend(self.parse(
                    prefix,
                    format!(
                        "failed to parse prefix for field `{}` in variant `{}`",
                        name, variant_name
                    ),
                    false,
                ));
            }

            let parser = attrs.parser.clone().unwrap_or(ty.clone());
            tokens.extend(quote! {let #name = });
            tokens.extend(self.parse(
                parser,
                format!(
                    "could not parse field `{}` in variant `{}`",
                    name, variant_name
                ),
                attrs.parser.is_some(),
            ));

            if let Some(suffix) = attrs.suffix.clone() {
                tokens.extend(self.parse(
                    suffix,
                    format!(
                        "failed to parse suffix for field `{}` in variant `{}`",
                        name, variant_name
                    ),
                    false,
                ));
            }

            output.extend(quote! {
                #name: #name.into(),
            })
        }

        if let Some(suffix) = self.attrs.suffix.clone() {
            tokens.extend(self.parse(
                suffix,
                format!("failed to parse suffix for enum `{}`", self.name),
                false,
            ));
        }

        let enum_name = &self.name;
        tokens.extend(quote! {
            Ok(#enum_name::#variant_name{
                #output
            })
        });

        self.parse_fn_impl.push(tokens)
    }

    fn add_tuple_parse(&mut self, variant_name: syn::Ident, unnamed: Vec<UnnamedField>) {
        let mut tokens = TokenStream::new();

        if let Some(prefix) = self.attrs.prefix.clone() {
            tokens.extend(self.parse(
                prefix,
                format!("failed to parse prefix for enum `{}`", self.name),
                false,
            ));
        }

        let mut output = TokenStream::new();

        for (i, field) in unnamed.into_iter().enumerate() {
            let UnnamedField { attrs, ty } = field;
            let name = format_ident!("field{}", i);

            if let Some(prefix) = attrs.prefix.clone() {
                tokens.extend(self.parse(
                    prefix,
                    format!(
                        "failed to parse prefix for field {} in variant `{}`",
                        i, variant_name
                    ),
                    false,
                ));
            }

            let parser = attrs.parser.clone().unwrap_or(ty.clone());
            tokens.extend(quote! {let #name = });
            tokens.extend(self.parse(
                parser,
                format!("could not parse field {} in variant `{}`", i, variant_name),
                attrs.parser.is_some(),
            ));

            if let Some(suffix) = attrs.suffix.clone() {
                tokens.extend(self.parse(
                    suffix,
                    format!(
                        "failed to parse suffix for field {} in variant `{}`",
                        i, variant_name
                    ),
                    false,
                ));
            }

            output.extend(quote! {
                #name.into(),
            })
        }

        if let Some(suffix) = self.attrs.suffix.clone() {
            tokens.extend(self.parse(
                suffix,
                format!("failed to parse suffix for enum `{}`", self.name),
                false,
            ));
        }

        let enum_name = &self.name;
        tokens.extend(quote! {
            Ok(#enum_name::#variant_name(
                #output
            ))
        });

        self.parse_fn_impl.push(tokens)
    }
}

impl ToTokens for EnumParse {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let EnumParse {
            fn_impl,
            parse_fn_names,
            parse_fn_impl,
            attrs: _,
            after_each: _,
            name,
            where_clause_types,
            args,
            parse_type,
        } = self;

        let error_message = format!("no variants of {} could be parsed", name);

        let where_clause = quote!{
            where #(
                #where_clause_types: ::nommy::Parse<#parse_type>,
            )*
        };

        tokens.extend(quote!{
            #[automatically_derived]
            impl <#parse_type, #(#args),*> ::nommy::Parse<#parse_type> for #name<#(#args),*> #where_clause {
                fn parse(input: &mut ::nommy::Buffer<impl ::std::iter::Iterator<Item=#parse_type>>) -> ::nommy::eyre::Result<Self> {
                    use ::nommy::eyre::WrapErr;

                    #fn_impl {
                        Err(::nommy::eyre::eyre!(#error_message))
                    }
                }
            }

            #[automatically_derived]
            impl<#(#args),*> #name<#(#args),*>where
            {
                #(
                    fn #parse_fn_names<#parse_type>(input: &mut ::nommy::Buffer<impl ::std::iter::Iterator<Item=#parse_type>>) -> ::nommy::eyre::Result<Self> #where_clause {
                        use ::nommy::eyre::WrapErr;
                        #parse_fn_impl
                    }
                )*
            }
        })
    }
}
