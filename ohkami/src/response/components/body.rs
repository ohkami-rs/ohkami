use serde::Serialize;

pub(crate) trait Text: Serialize {}
impl Text for String {}
impl<'text> Text for &'text str {}

pub(crate) trait Html: Serialize {}
impl Html for String {}
impl<'html> Html for &'html str {}
