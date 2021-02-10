use super::Enum;
use crate::{attr::GlobalAttr, fn_impl::FnImpl, parsers::NamedField};
use proc_macro2::TokenStream;
use quote::quote;

pub struct EnumVariantUnit {
    pub name: syn::Ident,
    pub attrs: GlobalAttr,
}

impl FnImpl<NamedField> for (&EnumVariantUnit, &Enum) {
    const TYPE: &'static str = "unit variant";
    fn fields(&self) -> &[NamedField] {
        &[]
    }
    fn name(&self) -> &syn::Ident {
        &self.0.name
    }
    fn generic(&self) -> &syn::Type {
        &self.1.generic
    }
    fn attrs(&self) -> &GlobalAttr {
        &self.0.attrs
    }
}

impl EnumVariantUnit {
    pub fn result(&self, enum_: &Enum) -> TokenStream {
        let enum_name = &enum_.name;
        let variant_name = &self.name;
        quote! {
            Ok(#enum_name::#variant_name)
        }
    }
}
