use super::Enum;
use crate::{
    attr::GlobalAttr,
    fn_impl::FnImpl,
    parsers::{FieldType, UnnamedField},
};
use proc_macro2::TokenStream;
use quote::quote;

pub struct EnumVariantUnnamed {
    pub name: syn::Ident,
    pub attrs: GlobalAttr,
    pub fields: Vec<UnnamedField>,
}

impl EnumVariantUnnamed {
    pub fn fn_impl<'a>(&'a self, enum_: &'a Enum) -> FnImpl<'a, UnnamedField> {
        let Self {
            name,
            attrs,
            fields,
        } = self;
        FnImpl {
            ty: "tuple variant",
            name,
            fields,
            attrs,
            generic: &enum_.generic,
        }
    }

    pub fn result(&self, enum_: &Enum) -> TokenStream {
        let names = self.fields.iter().enumerate().map(|(i, f)| f.name(i));
        let enum_name = &enum_.name;
        let variant_name = &self.name;
        quote! {
            Ok(#enum_name::#variant_name (#(
                #names,
            )*))
        }
    }
}
