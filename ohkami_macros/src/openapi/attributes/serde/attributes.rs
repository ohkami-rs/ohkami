//! We only have to do collect **valid** `#[serde(...)]` attribtues
//! because any errors are handled in serde-side
//! and we just need to intercept them.

use super::{Case, EqValue, Separatable};
use syn::{token, Ident};

#[derive(Default)]
pub(crate) struct ContainerAttributes {
    pub(crate) rename:            Separatable<String>,
    pub(crate) rename_all:        Separatable<Case>,
    pub(crate) rename_all_fields: Separatable<Case>,
    pub(crate) tag:               EqValue,
    pub(crate) content:           EqValue,
    pub(crate) untagged:          bool,
    pub(crate) default:           bool,
    pub(crate) transparent:       bool,
    pub(crate) from:              EqValue,
    pub(crate) try_from:          EqValue,
    pub(crate) into:              EqValue,
}
impl syn::parse::Parse for ContainerAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = ContainerAttributes::default();

        while let Ok(i) = input.parse::<Ident>() {
            match &*i.to_string() {
                "rename"            => this.rename            = input.parse()?,
                "rename_all"        => this.rename_all        = input.parse()?,
                "rename_all_fields" => this.rename_all_fields = input.parse()?,
                "tag"               => this.tag               = input.parse()?,
                "content"           => this.content           = input.parse()?,
                "untagged"          => this.untagged          = true,
                "default"           => this.default           = true,
                "transparent"       => this.transparent       = true,
                "from"              => this.from              = input.parse()?,
                "try_from"          => this.try_from          = input.parse()?,
                "into"              => this.into              = input.parse()?,
                _ => ()
            }

            if input.peek(token::Comma) {
                let _ = input.parse::<token::Comma>()?;
            }
        }

        Ok(this)
    }
}

#[derive(Default)]
pub(crate) struct FieldAttributes {
    pub(crate) rename:              Separatable<String>,
    pub(crate) alias:               EqValue,
    pub(crate) default:             bool,
    pub(crate) flatten:             bool,
    pub(crate) skip:                bool,
    pub(crate) skip_serializing:    bool,
    pub(crate) skip_deserializing:  bool,
    pub(crate) skip_serializing_if: EqValue,
}
impl syn::parse::Parse for FieldAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = FieldAttributes::default();

        while let Ok(i) = input.parse::<Ident>() {
            match &*i.to_string() {
                "rename"              => this.rename              = input.parse()?,
                "alias"               => this.alias               = input.parse()?,
                "default"             => this.default             = true,
                "flatten"             => this.flatten             = true,
                "skip"                => this.skip                = true,
                "skip_serializing"    => this.skip_serializing    = true,
                "skip_deserializing"  => this.skip_deserializing  = true,
                "skip_serializing_if" => this.skip_serializing_if = input.parse()?,
                _ => ()
            }

            if input.peek(token::Comma) {
                let _ = input.parse::<token::Comma>()?;
            }
        }

        Ok(this)
    }
}

#[derive(Default)]
pub(crate) struct VariantAttributes {
    pub(crate) rename:              Separatable<String>,
    pub(crate) alias:               EqValue,
    pub(crate) rename_all:          Separatable<Case>,
    pub(crate) skip:                bool,
    pub(crate) skip_serializing:    bool,
    pub(crate) skip_deserializing:  bool,
    pub(crate) skip_serializing_if: EqValue,
    pub(crate) other:               bool,
    pub(crate) untagged:            bool,
}
impl syn::parse::Parse for VariantAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = VariantAttributes::default();

        while let Ok(i) = input.parse::<Ident>() {
            match &*i.to_string() {
                "rename"              => this.rename              = input.parse()?,
                "alias"               => this.alias               = input.parse()?,
                "rename_all"          => this.rename_all          = input.parse()?,
                "skip"                => this.skip                = true,
                "skip_serializing"    => this.skip_serializing    = true,
                "skip_deserializing"  => this.skip_deserializing  = true,
                "skip_serializing_if" => this.skip_serializing_if = input.parse()?,
                "other"               => this.other               = true,
                "untagged"            => this.untagged            = true,
                _ => ()
            }

            if input.peek(token::Comma) {
                let _ = input.parse::<token::Comma>()?;
            }
        }

        Ok(this)
    }
}

