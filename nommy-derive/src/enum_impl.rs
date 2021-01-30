use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
    attr::{FieldAttr, GlobalAttr},
    parsers::{
        path_from_idents, FieldPeeker, NamedField, NamedFieldParser, UnnamedField,
        UnnamedFieldParser,
    },
};

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
        };

        peek_impl.enrich(input.fields);

        peek_impl
    }

    fn enrich(&mut self, fields: Vec<EnumField>) {
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

    fn add_struct_peek(&mut self, named: Vec<NamedField>) {
        self.peek_fn_impl.push(
            FieldPeeker {
                attrs: &self.attrs,
                peek_type: &self.peek_type,
                fields: named
                    .into_iter()
                    .map(|field| {
                        let NamedField { attrs, name: _, ty } = field;
                        UnnamedField { attrs, ty }
                    })
                    .collect(),
            }
            .to_tokens(&mut self.where_clause_types),
        );
    }

    fn add_tuple_peek(&mut self, unnamed: Vec<UnnamedField>) {
        self.peek_fn_impl.push(
            FieldPeeker {
                attrs: &self.attrs,
                peek_type: &self.peek_type,
                fields: unnamed,
            }
            .to_tokens(&mut self.where_clause_types),
        );
    }
}

impl ToTokens for EnumPeek {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let EnumPeek {
            fn_impl,
            peek_fn_names,
            peek_fn_impl,
            attrs: _,
            name,
            where_clause_types,
            args,
            peek_type,
        } = self;

        let where_clause = quote! {
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
        };

        parse_impl.enrich(input.fields);

        parse_impl
    }

    fn enrich(&mut self, fields: Vec<EnumField>) {
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

    fn add_struct_parse(&mut self, variant_name: syn::Ident, named: Vec<NamedField>) {
        self.parse_fn_impl.push(
            NamedFieldParser {
                struct_path: path_from_idents(vec![self.name.clone(), variant_name]),
                attrs: &self.attrs,
                parse_type: &self.parse_type,
                fields: named,
            }
            .to_tokens(&mut self.where_clause_types),
        );
    }

    fn add_tuple_parse(&mut self, variant_name: syn::Ident, unnamed: Vec<UnnamedField>) {
        self.parse_fn_impl.push(
            UnnamedFieldParser {
                tuple_path: path_from_idents(vec![self.name.clone(), variant_name]),
                attrs: &self.attrs,
                parse_type: &self.parse_type,
                fields: unnamed,
            }
            .to_tokens(&mut self.where_clause_types),
        );
    }
}

impl ToTokens for EnumParse {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let EnumParse {
            fn_impl,
            parse_fn_names,
            parse_fn_impl,
            attrs: _,
            name,
            where_clause_types,
            args,
            parse_type,
        } = self;

        let error_message = format!("no variants of {} could be parsed", name);

        let where_clause = quote! {
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
