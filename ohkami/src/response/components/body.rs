use serde::Serialize;

pub(crate) trait Text: Serialize {fn as_str(&self) -> &str;}
impl Text for String {fn as_str(&self) -> &str {&self}}
impl<'text> Text for &'text str {fn as_str(&self) -> &str {self}}

pub(crate) trait Html: Serialize {fn as_str(&self) -> &str;}
impl Html for String {fn as_str(&self) -> &str {&self}}
impl<'html> Html for &'html str {fn as_str(&self) -> &str {self}}

pub(crate) trait Json: Serialize {fn as_str(&self) -> &str;}
impl<S: Serialize> Json for S {
    default fn as_str(&self) -> &str {
        &serde_json::to_string(self).unwrap()
    }
}
impl<'j> Json for &'j str {
    fn as_str(&self) -> &str {
        self
    }
}
