use proc_macro2::TokenStream;
use quote::quote;
use crate::{attr::GlobalAttr, fn_impl::FnImpl, parsers::NamedField};
use super::Enum;

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
    fn result(&self) -> TokenStream {
        let names = self.0.fields.iter().map(|f| &f.name);
        let enum_name = &self.1.name;
        let variant_name = &self.0.name;
        quote! {
            Ok(#enum_name::#variant_name {#(
                #names: #names.into(),
            )*})
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
