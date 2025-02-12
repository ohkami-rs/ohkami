use syn::{token, Ident, LitStr};

#[derive(Default)]
pub(crate) struct ContainerAttributes {
    pub(crate) component: ComponentConfig,
}
#[derive(Default)]
pub(crate) struct ComponentConfig {
    pub(crate) yes:  bool,
    pub(crate) name: Option<String>,
}
impl syn::parse::Parse for ContainerAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = ContainerAttributes::default();
        while let Ok(i) = input.parse::<Ident>() {
            match &*i.to_string() {
                "component" => {
                    this.component.yes = true;
                    if input.peek(token::Eq) {
                        let _ = input.parse::<token::Eq>()?;
                        this.component.name = Some(input.parse::<LitStr>()?.value());
                    }
                },
                other => return Err(syn::Error::new(i.span(), format!("\
                    Unexpected specifier `{other}` in #[openapi]. Expected one of: \n\
                    - component \n\
                ")))
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
    pub(crate) schema_with: Option<String>,
}
impl syn::parse::Parse for FieldAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = FieldAttributes::default();
        while let Ok(i) = input.parse::<Ident>() {
            match &*i.to_string() {
                "schema_with" => {
                    let _ = input.parse::<token::Eq>()?;
                    let l = input.parse::<LitStr>()?;
                    this.schema_with = Some(l.value());
                },
                other => return Err(syn::Error::new(i.span(), format!("\
                    Unexpected specifier `{other}` in #[openapi]. Expected one of: \n\
                    - schema_with \n\
                ")))
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
    pub(crate) schema_with: Option<String>,
}
impl syn::parse::Parse for VariantAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = VariantAttributes::default();
        while let Ok(i) = input.parse::<Ident>() {
            match &*i.to_string() {
                "schema_with" => {
                    let _ = input.parse::<token::Eq>()?;
                    let l = input.parse::<LitStr>()?;
                    this.schema_with = Some(l.value());
                },
                other => return Err(syn::Error::new(i.span(), format!("\
                    Unexpected specifier `{other}` in #[openapi]. Expected one of: \n\
                    - schema_with \n\
                ")))
            }
            if input.peek(token::Comma) {
                let _ = input.parse::<token::Comma>()?;
            }
        }
        Ok(this)
    }
}
