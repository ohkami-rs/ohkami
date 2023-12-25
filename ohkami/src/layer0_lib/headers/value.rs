use std::borrow::Cow;


pub type HeaderValue = Cow<'static, str>;

pub trait IntoHeaderValue {
    fn into_header_value(self) -> HeaderValue;
} impl<C: Into<Cow<'static, str>>> IntoHeaderValue for C {
    #[inline(always)] fn into_header_value(self) -> HeaderValue {
        self.into()
    }
}

#[cfg(test)] fn assert_impls() {
    fn impls_into_header_value<T: IntoHeaderValue>() {}

    impls_into_header_value::<String>();
    impls_into_header_value::<&'static str>();
    impls_into_header_value::<&str>();
}
