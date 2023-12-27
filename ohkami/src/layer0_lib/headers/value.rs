use std::borrow::Cow;


pub struct HeaderValue {
    value: Cow<'static, str>
}

pub trait IntoHeaderValue {
    fn into_header_value(self) -> HeaderValue;
} impl<C: Into<Cow<'static, str>>> IntoHeaderValue for C {
    #[inline(always)] fn into_header_value(self) -> HeaderValue {
        HeaderValue {
            value: self.into(),
        }
    }
}

#[cfg(test)] fn assert_impls() {
    fn impls_into_header_value<T: IntoHeaderValue>() {}

    impls_into_header_value::<String>();
    impls_into_header_value::<&'static str>();
    impls_into_header_value::<&str>();
}

impl HeaderValue {
    #[inline] pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn append(&mut self, next_value: Self) {
        match &mut self.value {
            Cow::Owned(v) => {v.push(','); v.push_str(&next_value.value)}
            _ => unsafe {std::hint::unreachable_unchecked()}
        }
    }
}
