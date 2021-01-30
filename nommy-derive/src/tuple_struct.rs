use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
    attr::{FieldAttr, GlobalAttr},
    parsers::UnnamedField,
    parsers::{path_from_ident, FieldPeeker, UnnamedFieldParser},
};

#[derive(Clone)]
pub struct TupleStructInput {
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub args: Vec<syn::Ident>,
    pub fields: Vec<UnnamedField>,
}

impl TupleStructInput {
    pub fn new(
        name: syn::Ident,
        generics: syn::Generics,
        attrs: Vec<syn::Attribute>,
        fields: syn::FieldsUnnamed,
    ) -> Self {
        let args = generics.type_params().cloned().map(|tp| tp.ident).collect();

        let fields = fields
            .unnamed
            .into_iter()
            .map(|field| {
                let syn::Field { attrs, ty, .. } = field;
                let attrs = FieldAttr::parse_attrs(attrs);
                UnnamedField { attrs, ty }
            })
            .collect();

        let attrs = GlobalAttr::parse_attrs(attrs);

        TupleStructInput {
            attrs,
            name,
            args,
            fields,
        }
    }

    pub fn process(self) -> TupleStructOutput {
        TupleStructOutput {
            peek_impl: TupleStructPeek::new(self.clone()),
            parse_impl: TupleStructParse::new(self),
        }
    }
}

pub struct TupleStructOutput {
    peek_impl: TupleStructPeek,
    parse_impl: TupleStructParse,
}

impl ToTokens for TupleStructOutput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let TupleStructOutput {
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

pub struct TupleStructPeek {
    pub fn_impl: TokenStream,
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub where_clause_types: Vec<syn::Type>,
    pub args: Vec<syn::Ident>,
    pub peek_type: syn::Ident,
    pub after_each: TokenStream,
}

impl TupleStructPeek {
    fn new(input: TupleStructInput) -> Self {
        let peek_type = format_ident!("__PeekType");

        let mut peek_impl = TupleStructPeek {
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

    fn enrich(&mut self, fields: Vec<UnnamedField>) {
        self.fn_impl.extend(
            FieldPeeker {
                attrs: &self.attrs,
                peek_type: &self.peek_type,
                fields,
            }
            .to_tokens(&mut self.where_clause_types),
        );
    }
}

impl ToTokens for TupleStructPeek {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let TupleStructPeek {
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

pub struct TupleStructParse {
    pub fn_impl: TokenStream,
    pub attrs: GlobalAttr,
    pub name: syn::Ident,
    pub where_clause_types: Vec<syn::Type>,
    pub args: Vec<syn::Ident>,
    pub parse_type: syn::Ident,
    pub after_each: TokenStream,
}

impl TupleStructParse {
    fn new(input: TupleStructInput) -> Self {
        let parse_type = format_ident!("__ParseType");

        let mut parse_impl = TupleStructParse {
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

    fn enrich(&mut self, fields: Vec<UnnamedField>) {
        self.fn_impl.extend(
            UnnamedFieldParser {
                tuple_path: path_from_ident(self.name.clone()),
                attrs: &self.attrs,
                parse_type: &self.parse_type,
                fields,
            }
            .to_tokens(&mut self.where_clause_types),
        );
    }
}

impl ToTokens for TupleStructParse {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let TupleStructParse {
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
