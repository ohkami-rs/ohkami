use std::borrow::Cow;


pub trait AsStr: serde::Serialize {
    fn as_str(&self) -> &str;
}
impl AsStr for String {fn as_str(&self) -> &str {&self}}
impl AsStr for &str {fn as_str(&self) -> &str {self}}
impl AsStr for () {fn as_str(&self) -> &str {""}}


pub trait IntoCow<'l> {
    fn into_cow(self) -> Cow<'l, str>;
}
impl IntoCow<'static> for &'static str {fn into_cow(self) -> Cow<'static, str> {Cow::Borrowed(self)}}
impl IntoCow<'static> for String {fn into_cow(self) -> Cow<'static, str> {Cow::Owned(self)}}
