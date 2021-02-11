use std::convert::TryFrom;

use quote::format_ident;

use crate::attr::FieldAttr;

pub struct NamedField {
    pub attrs: FieldAttr,
    pub name: syn::Ident,
    pub ty: syn::Type,
}

impl TryFrom<syn::Field> for NamedField {
    type Error = syn::Error;
    fn try_from(field: syn::Field) -> syn::Result<Self> {
        let syn::Field {
            ident, attrs, ty, ..
        } = field;
        let attrs = FieldAttr::parse_attrs(attrs)?;
        Ok(NamedField {
            attrs,
            name: ident.unwrap(),
            ty,
        })
    }
}

impl TryFrom<syn::Field> for UnnamedField {
    type Error = syn::Error;
    fn try_from(field: syn::Field) -> syn::Result<Self> {
        let syn::Field { attrs, ty, .. } = field;
        let attrs = FieldAttr::parse_attrs(attrs)?;
        Ok(UnnamedField { attrs, ty })
    }
}

pub struct UnnamedField {
    pub attrs: FieldAttr,
    pub ty: syn::Type,
}

pub trait FieldType {
    fn ty(&self) -> &syn::Type;
    fn name(&self, i: usize) -> syn::Ident;
    fn attrs(&self) -> &FieldAttr;
}

impl FieldType for NamedField {
    fn ty(&self) -> &syn::Type {
        &self.ty
    }
    fn name(&self, _: usize) -> syn::Ident {
        self.name.clone()
    }
    fn attrs(&self) -> &FieldAttr {
        &self.attrs
    }
}

impl FieldType for UnnamedField {
    fn ty(&self) -> &syn::Type {
        &self.ty
    }
    fn name(&self, i: usize) -> syn::Ident {
        format_ident!("elem{}", i)
    }
    fn attrs(&self) -> &FieldAttr {
        &self.attrs
    }
}
