use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::attr::{FieldAttr, GlobalAttr, IgnoreWS};

#[derive(Clone)]
pub struct NamedStructInput {
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub fields: Vec<NamedField>,
}

impl NamedStructInput {
    pub fn new(
        name: syn::Ident,
        generics: syn::Generics,
        attrs: Vec<syn::Attribute>,
        fields: syn::FieldsNamed,
    ) -> Self {
        let args = generics.type_params().cloned().map(|tp| tp.ident).collect();

        let fields = fields
            .named
            .into_iter()
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
            .collect();

        let attrs = GlobalAttr::parse_attrs(attrs);

        NamedStructInput {
            attrs,
            name,
            args,
            fields,
        }
    }

    pub fn process(self) -> NamedStructOutput {
        NamedStructOutput {
            peek_impl: NamedStructPeek::new(self.clone()),
            parse_impl: NamedStructParse::new(self),
        }
    }
}

pub struct NamedStructOutput {
    peek_impl: NamedStructPeek,
    parse_impl: NamedStructParse,
}

impl ToTokens for NamedStructOutput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NamedStructOutput {
            peek_impl,
            parse_impl,
        } = self;

        peek_impl.to_tokens(tokens);
        parse_impl.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct NamedField {
    pub attrs: FieldAttr,
    pub name: syn::Ident,
    pub ty: syn::Type,
}

pub struct NamedStructPeek {
    pub fn_impl: TokenStream,
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub where_clause_types: Vec<syn::Type>,
    pub args: Vec<syn::Ident>,
    pub peek_type: syn::Ident,
    pub after_each: TokenStream,
}

impl NamedStructPeek {
    fn new(input: NamedStructInput) -> Self {
        let peek_type = format_ident!("__PeekType");

        let mut peek_impl = NamedStructPeek {
            fn_impl: Default::default(),
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

    fn enrich(&mut self, fields: Vec<NamedField>) {
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

        if let Some(prefix) = self.attrs.prefix.clone() {
            self.add_peek(prefix)
        }

        for field in fields {
            let NamedField { attrs, name: _, ty } = field;

            if let Some(prefix) = attrs.prefix {
                self.add_peek(prefix)
            }

            let parser = attrs.parser.unwrap_or(ty);
            self.add_peek(parser);

            if let Some(suffix) = attrs.suffix {
                self.add_peek(suffix)
            }
        }

        if let Some(suffix) = self.attrs.suffix.clone() {
            self.add_peek(suffix)
        }
    }

    fn add_peek(&mut self, peeker: syn::Type) {
        self.fn_impl.extend(self.peek_tokens(&peeker));
        self.fn_impl.extend(self.after_each.clone());
        self.where_clause_types.push(peeker.clone());
    }

    fn peek_tokens(&self, peeker: &syn::Type) -> TokenStream {
        let peek_type = &self.peek_type;
        quote! {
            if !(<#peeker as ::nommy::Peek<#peek_type>>::peek(input)) { return false }
        }
    }
}

impl ToTokens for NamedStructPeek {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NamedStructPeek {
            fn_impl,
            attrs: _,
            after_each: _,
            name,
            where_clause_types,
            args,
            peek_type,
        } = self;

        tokens.extend(quote!{
            #[automatically_derived]
            impl <#peek_type, #(#args),*> ::nommy::Peek<#peek_type> for #name<#(#args),*>
            where #(
                #where_clause_types: ::nommy::Peek<#peek_type>,
            )* {
                fn peek(input: &mut ::nommy::Cursor<impl ::std::iter::Iterator<Item=#peek_type>>) -> bool {
                    #fn_impl
                    true
                }
            }
        })
    }
}

pub struct NamedStructParse {
    pub fn_impl: TokenStream,
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub where_clause_types: Vec<syn::Type>,
    pub args: Vec<syn::Ident>,
    pub parse_type: syn::Ident,
    pub after_each: TokenStream,
}

impl NamedStructParse {
    fn new(input: NamedStructInput) -> Self {
        let parse_type = format_ident!("__ParseType");

        let mut parse_impl = NamedStructParse {
            fn_impl: Default::default(),
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

    fn enrich(&mut self, fields: Vec<NamedField>) {
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

        if let Some(prefix) = self.attrs.prefix.clone() {
            self.add_parse(
                prefix,
                format!("failed to parse prefix for struct `{}`", self.name),
                false,
            );
        }

        let mut output = TokenStream::new();

        for field in fields {
            let NamedField { attrs, name, ty } = field;

            if let Some(prefix) = attrs.prefix.clone() {
                self.add_parse(
                    prefix,
                    format!("failed to parse prefix for field `{}`", name),
                    false,
                );
            }

            let parser = attrs.parser.clone().unwrap_or(ty.clone());
            self.fn_impl.extend(quote! {let #name = });
            self.add_parse(
                parser,
                format!("could not parse field `{}`", name),
                attrs.parser.is_some(),
            );

            if let Some(suffix) = attrs.suffix.clone() {
                self.add_parse(
                    suffix,
                    format!("failed to parse suffix for field `{}`", name),
                    false,
                );
            }

            output.extend(quote! {
                #name: #name.into(),
            })
        }

        if let Some(suffix) = self.attrs.suffix.clone() {
            self.add_parse(
                suffix,
                format!("failed to parse suffix for struct `{}`", self.name),
                false,
            );
        }

        let name = &self.name;
        self.fn_impl.extend(quote! {
            Ok(#name{#output})
        })
    }

    fn add_parse(&mut self, parser: syn::Type, error: String, process: bool) {
        self.fn_impl
            .extend(self.parse_tokens(&parser, error, process));
        self.fn_impl.extend(self.after_each.clone());
        self.where_clause_types.push(parser.clone());
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

impl ToTokens for NamedStructParse {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NamedStructParse {
            fn_impl,
            attrs: _,
            after_each: _,
            name,
            where_clause_types,
            args,
            parse_type,
        } = self;

        tokens.extend(quote!{
            #[automatically_derived]
            impl <#parse_type, #(#args),*> ::nommy::Parse<#parse_type> for #name<#(#args),*>
            where #(
                #where_clause_types: ::nommy::Parse<#parse_type>,
            )* {
                fn parse(input: &mut ::nommy::Buffer<impl ::std::iter::Iterator<Item=#parse_type>>) -> ::nommy::eyre::Result<Self> {
                    use ::nommy::eyre::WrapErr;
                    #fn_impl
                }
            }
        })
    }
}
