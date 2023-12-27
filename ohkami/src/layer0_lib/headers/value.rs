use std::borrow::Cow;


pub struct HeaderValue {
    value: Cow<'static, [u8]>
}

pub trait IntoHeaderValue {
    fn into_header_value(self) -> HeaderValue;
} impl<C: Into<Cow<'static, [u8]>>> IntoHeaderValue for C {
    #[inline(always)] fn into_header_value(self) -> HeaderValue {
        HeaderValue {
            value: self.into(),
        }
    }
}

#[cfg(test)] fn assert_impls() {
    fn impls_into_header_value<T: IntoHeaderValue>() {}

    impls_into_header_value::<&[u8]>();
    impls_into_header_value::<Vec<u8>>();
}

impl HeaderValue {
    #[inline] pub fn as_bytes(&self) -> &[u8] {
        &self.value
    }

    pub fn append(&mut self, next_value: Self) {
        match &mut self.value {
            Cow::Owned(v) => {v.push(b','); v.copy_from_slice(&next_value.value)}
            _ => unsafe {std::hint::unreachable_unchecked()}
        }
    }
}
