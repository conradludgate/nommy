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

impl FnImpl<UnnamedField> for (&EnumVariantUnnamed, &Enum) {
    const TYPE: &'static str = "tuple variant";
    fn fields(&self) -> &[UnnamedField] {
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

impl EnumVariantUnnamed {
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
