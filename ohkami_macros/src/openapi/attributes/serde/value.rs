use syn::{token, LitStr, Ident};
use proc_macro2::Span;

////////////////////////////////////////////////////////////

pub(crate) struct EqValue<T: From<String> = String>(
    Option<T>
);

impl<T: From<String>> Default for EqValue<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: From<String>> syn::parse::Parse for EqValue<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.peek(token::Eq).then(|| {
            let _ = input.parse::<token::Eq>()?;
            let l = input.parse::<LitStr>()?;
            Ok(l.value().into())
        }).transpose().map(Self)
    }
}

impl<T: From<String>> std::ops::Deref for EqValue<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

////////////////////////////////////////////////////////////

#[derive(Clone)]
pub(crate) struct Separatable<T: From<String>> {
    span:        Span,
    serialize:   Option<T>,
    deserialize: Option<T>,
}

impl<T: From<String>> Default for Separatable<T> {
    fn default() -> Self {
        Self {
            span:        Span::call_site(),
            serialize:   None,
            deserialize: None,
        }
    }
}

impl<T: From<String>> syn::parse::Parse for Separatable<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut span        = Span::call_site();
        let mut serialize   = None;
        let mut deserialize = None;

        if input.peek(token::Eq) {
            let _ = input.parse::<token::Eq>()?;
            let l = input.parse::<LitStr>()?;
            
            span        = l.span();
            serialize   = Some(l.value().into());
            deserialize = Some(l.value().into());

        } else if input.peek(token::Brace) {
            let b; syn::braced!(b in input);
            while let Ok(i) = b.parse::<Ident>() {
                let _ = b.parse::<token::Eq>()?;
                let l = b.parse::<LitStr>()?;

                span = l.span();
                if i == "serialize" {
                    serialize = Some(l.value().into())
                } else if i == "deserialize" {
                    deserialize = Some(l.value().into())
                }

                if b.peek(token::Comma) {
                    let _ = b.parse::<token::Comma>()?;
                }
            }
        }

        Ok(Self { span, serialize, deserialize })
    }
}

impl<T: From<String>> Separatable<T>
where
    T: PartialEq
{
    pub(crate) fn value(&self) -> syn::Result<Option<(Span, &T)>> {
        match (&self.serialize, &self.deserialize) {
            (None,    None   )           => Ok(None),
            (Some(s), None   )           => Ok(Some((self.span.clone(), s))),
            (None,    Some(d))           => Ok(Some((self.span.clone(), d))),
            (Some(s), Some(d)) if s == d => Ok(Some((self.span.clone(), s))),
            _ => Err(syn::Error::new(
                self.span.clone(), "#[derive(Schema)] doesn't support \
                #[serde(rename())] with both `serialize = ...` and `deserialize = ...`"
            ))
        }
    }
}
