mod openapi;
mod serde;

#[derive(Default)]
pub(super) struct ContainerAttributes {
    pub(super) openapi: attributes::openapi::ContainerAttributes,
    pub(super) serde:   attribtues::serde::ContainerAttributes,
}
impl ContainerAttributes {
    fn new(attrs: &[Attribute]) -> Self {
        let mut this = Default::default();
        for a in attrs {
            let Ok(a) = a.meta.require_list() else {continue};
            if a.path.get_ident().is_some_and(|i| i == "openapi") {
                this.openapi = a.parse_args()?;
            }
            if a.path.get_ident().is_some_and(|i| i == "serde") {
                this.serde = a.parse_args()?;
            }
        }
        this
    }
}

#[derive(Default)]
pub(super) struct FieldAttributes {
    pub(super) openapi: attributes::openapi::FieldAttributes,
    pub(super) serde:   attribtues::serde::FieldAttributes,
}
impl FieldAttributes {
    fn new(attrs: &[Attribute]) -> Self {
        let mut this = Default::default();
        for a in attrs {
            let Ok(a) = a.meta.require_list() else {continue};
            if a.path.get_ident().is_some_and(|i| i == "openapi") {
                this.openapi = a.parse_args()?;
            }
            if a.path.get_ident().is_some_and(|i| i == "serde") {
                this.serde = a.parse_args()?;
            }
        }
        this
    }
}

#[derive(Default)]
pub(super) struct VariantAttributes {
    pub(super) openapi: attributes::openapi::VariantAttributes,
    pub(super) serde:   attribtues::serde::VariantAttributes,
}
impl VariantAttributes {
    fn new(attrs: &[Attribute]) -> Self {
        let mut this = Default::default();
        for a in attrs {
            let Ok(a) = a.meta.require_list() else {continue};
            if a.path.get_ident().is_some_and(|i| i == "openapi") {
                this.openapi = a.parse_args()?;
            }
            if a.path.get_ident().is_some_and(|i| i == "serde") {
                this.serde = a.parse_args()?;
            }
        }
        this
    }
}
