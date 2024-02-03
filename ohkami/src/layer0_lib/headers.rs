pub fn append(value: impl Into<std::borrow::Cow<'static, str>>) -> Append {
    Append(value.into())
}

pub struct Append(pub(crate) std::borrow::Cow<'static, str>);
