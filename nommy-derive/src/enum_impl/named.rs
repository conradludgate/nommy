use super::Enum;
use crate::{attr::GlobalAttr, fn_impl::FnImpl, parsers::NamedField};
use proc_macro2::TokenStream;
use quote::quote;

pub struct EnumVariantNamed {
    pub name: syn::Ident,
    pub attrs: GlobalAttr,
    pub fields: Vec<NamedField>,
}

impl FnImpl<NamedField> for (&EnumVariantNamed, &Enum) {
    const TYPE: &'static str = "struct variant";
    fn fields(&self) -> &[NamedField] {
        &self.0.fields
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

impl EnumVariantNamed {
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
