#[derive(Default)]
pub(crate) struct EqValue<T: From<String> = String>(
    Option<T>
);

impl<T: From<String>> Parse for EqValue {
    fn parse(input: syn::ParseStream) -> syn::Result<Self> {
        input.peek(token::Eq).then(|| {
            let _ = input.peek::<token::Eq>()?;
            let l = input.peek::<LitStr>()?;
            Ok(Self(l.value().into()))
        }).transpose()
    }
}

impl<T: From<String>> std::ops::Deref for EqValue<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/////////////////////////////////////////////////////////////////////

#[derive(Default, Clone)]
pub(crate) struct Separatable<T: From<String>> {
    span:        syn::Span,
    serailize:   Option<T>,
    deserialize: Option<T>,
}

impl<T: From<String>> Parse for Separatable {
    fn parse(input: syn::ParseStream) -> syn::Result<Self> {
        let mut span        = syn::Span::call_site();
        let mut serailize   = None;
        let mut deserialize = None;

        if input.peek(token::Eq) {
            let _ = input.parse::<token::Eq>()?;
            let l = input.parse::<LitStr>()?;
            
            span        = l.span();
            serailize   = Some(l.value().into());
            deserialize = Some(l.value().into());

        } else if input.peek(token::Brace) {
            let b; syn::braced!(b in input);
            while let Ok(i) = b.parse::<Ident>() {
                let _ = b.parse::<token::Eq>()?;
                let l = b.parse::<LitStr>()?;

                span = l.span();
                if i == "serialize" {
                    serailize = Some(l.value())
                } else if i == "deserialize" {
                    deserialize = Some(l.value())
                }

                if b.peek(token::Comma) {
                    let _ = b.parse::<token::Comma>()?;
                }
            }
        }

        Ok(Self { span, serialize, deserialize })
    }
}

impl<T: From<String>> Separatable<T> {
    pub(crate) fn is_empty(&self) -> bool {
        self.serailize.is_none() && self.deserialize.is_none()
    }

    pub(crate) fn value(&self) -> syn::Result<Option<(syn::Span, &T)>> {
        match (&self.serialize, &self.deserialize) {
            (None,    None   )           => Ok(None),
            (Some(s), None   )           => Ok(Some(s.span(), s)),
            (None,    Some(d))           => Ok(Some(d.span(), d)),
            (Some(s), Some(d)) if s == d => Ok(Some(s.span(), s)),
            _ => Err(syn::Error::new(
                self.span.clone(), "#[derive(Schema)] doesn't support \
                #[serde(rename())] with both `serialize = ...` and `deserialize = ...`"
            ))
        }
    }
}
