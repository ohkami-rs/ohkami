mod openapi;
mod serde;

use syn::Attribute;

#[derive(Default)]
pub(super) struct ContainerAttributes {
    pub(super) openapi: openapi::ContainerAttributes,
    pub(super) serde:   serde::ContainerAttributes,
}
impl ContainerAttributes {
    pub(super) fn new(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut this = ContainerAttributes::default();
        for a in attrs {
            let Ok(a) = a.meta.require_list() else {continue};
            if a.path.get_ident().is_some_and(|i| i == "openapi") {
                this.openapi = a.parse_args()?;
            }
            if a.path.get_ident().is_some_and(|i| i == "serde") {
                this.serde = a.parse_args()?;
            }
        }
        Ok(this)
    }
}

#[derive(Default)]
pub(super) struct FieldAttributes {
    pub(super) openapi: openapi::FieldAttributes,
    pub(super) serde:   serde::FieldAttributes,
}
impl FieldAttributes {
    pub(super) fn new(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut this = FieldAttributes::default();
        for a in attrs {
            let Ok(a) = a.meta.require_list() else {continue};
            if a.path.get_ident().is_some_and(|i| i == "openapi") {
                this.openapi = a.parse_args()?;
            }
            if a.path.get_ident().is_some_and(|i| i == "serde") {
                this.serde = a.parse_args()?;
            }
        }
        Ok(this)
    }
}

#[derive(Default)]
pub(super) struct VariantAttributes {
    pub(super) openapi: openapi::VariantAttributes,
    pub(super) serde:   serde::VariantAttributes,
}
impl VariantAttributes {
    pub(super) fn new(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut this = VariantAttributes::default();
        for a in attrs {
            let Ok(a) = a.meta.require_list() else {continue};
            if a.path.get_ident().is_some_and(|i| i == "openapi") {
                this.openapi = a.parse_args()?;
            }
            if a.path.get_ident().is_some_and(|i| i == "serde") {
                this.serde = a.parse_args()?;
            }
        }
        Ok(this)
    }
}
