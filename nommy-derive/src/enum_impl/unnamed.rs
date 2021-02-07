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
    fn result(&self) -> TokenStream {
        let names = self.0.fields.iter().enumerate().map(|(i, f)| f.name(i));
        let enum_name = &self.1.name;
        let variant_name = &self.0.name;
        quote! {
            Ok(#enum_name::#variant_name (#(
                #names,
            )*))
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
