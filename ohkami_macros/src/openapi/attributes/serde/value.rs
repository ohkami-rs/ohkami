#[derive(Default)]
pub(crate) struct EqValue<T: From<String> = String>(
    Option<T>
);

impl<T: From<String>> std::ops::Deref for EqValue<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: From<String>> Parse for EqValue {
    fn parse(input: syn::ParseStream) -> syn::Result<Self> {
        input.peek(token::Eq).then(|| {
            let _ = input.peek::<token::Eq>()?;
            let l = input.peek::<LitStr>()?;
            Ok(Self(l.value().into()))
        }).transpose()
    }
}

/////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub(crate) struct Separatable<T: From<String>> {
    pub(crate) serailize:   Option<T>,
    pub(crate) deserialize: Option<T>,
}

impl<T: From<String>> Parse for Separatable {
    fn parse(input: syn::ParseStream) -> syn::Result<Self> {
        let mut serailize   = None;
        let mut deserialize = None;

        if input.peek(token::Eq) {
            let _ = input.parse::<token::Eq>()?;
            let t = input.parse::<LitStr>()?.value().into();

            serailize   = Some(t.clone());
            deserialize = Some(t);

        } else if input.peek(token::Brace) {
            let b; syn::braced!(b in input);
            while let Ok(i) = b.parse::<Ident>() {
                let _ = b.parse::<token::Eq>()?;
                let t = let t = b.parse::<LitStr>()?.value().into();

                if i == "serialize" {
                    serailize = Some(t)
                } else if i == "deserialize" {
                    deserialize = Some(t)
                }

                if b.peek(token::Comma) {
                    let _ = b.parse::<token::Comma>()?;
                }
            }
        }

        Ok(Self { serialize, deserialize })
    }
}
