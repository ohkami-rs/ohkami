pub(crate) mod client;
pub(crate) mod server;


pub fn append(value: impl Into<std::borrow::Cow<'static, str>>) -> Append {
    Append(value.into())
}
pub struct Append(std::borrow::Cow<'static, str>);
