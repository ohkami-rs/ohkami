use serde::Serialize;

pub(crate) trait Text: Serialize {}
impl Text for String {}
impl<'text> Text for &'text str {}


