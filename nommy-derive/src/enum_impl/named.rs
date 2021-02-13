use super::Enum;
use crate::{attr::GlobalAttr, fn_impl::FnImpl, parsers::NamedField};
use proc_macro2::TokenStream;
use quote::quote;

pub struct EnumVariantNamed {
    pub name: syn::Ident,
    pub attrs: GlobalAttr,
    pub fields: Vec<NamedField>,
}

impl EnumVariantNamed {
    pub fn fn_impl<'a>(&'a self, enum_: &'a Enum) -> FnImpl<'a, NamedField> {
        let Self {
            name,
            attrs,
            fields,
        } = self;

        FnImpl {
            ty: "struct variant",
            name,
            fields,
            attrs,
            generic: &enum_.generic,
        }
    }

    pub fn result(&self, enum_: &Enum) -> TokenStream {
        let names = self.fields.iter().map(|f| &f.name);
        let enum_name = &enum_.name;
        let variant_name = &self.name;
        quote! {
            Ok(#enum_name::#variant_name {#(
                #names: #names,
            )*})
        }
    }
}
