pub trait AsStr: serde::Serialize {
    fn as_str(&self) -> &str;
}
impl AsStr for String {fn as_str(&self) -> &str {&self}}
impl AsStr for &str {fn as_str(&self) -> &str {self}}
impl AsStr for () {fn as_str(&self) -> &str {""}}
