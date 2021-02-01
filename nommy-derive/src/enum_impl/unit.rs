use proc_macro2::TokenStream;
use quote::quote;
use crate::{attr::GlobalAttr, fn_impl::FnImpl, parsers::NamedField};
use super::Enum;

pub struct EnumVariantUnit {
    pub name: syn::Ident,
    pub attrs: GlobalAttr,
}

impl FnImpl<NamedField> for (&EnumVariantUnit, &Enum) {
    const TYPE: &'static str = "unit variant";
    fn fields(&self) -> &[NamedField] {
        &[]
    }
    fn result(&self) -> TokenStream {
        let enum_name = &self.1.name;
        let variant_name = &self.0.name;
        quote! {
            Ok(#enum_name::#variant_name)
        }
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
