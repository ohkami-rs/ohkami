#![cfg(feature="openapi")]

use super::super::{Fang, FangProc};
use std::path::PathBuf;


pub struct OpenAPI {
    pub(crate) file_path: PathBuf,
}

impl OpenAPI {
    pub fn json(file_path: impl Into<PathBuf>) -> Self {
        Self { file_path: file_path.into() }
    }
}

impl<Inner: FangProc> Fang<Inner> for OpenAPI {
    type Proc = Inner;
    fn chain(&self, inner: Inner) -> Self::Proc {
        crate::CONFIG.openapi_filepath()
            .set(self.file_path.clone())
            .expect("[OpenAPI] Unexpected multiple `OpenAPI`s in a `Ohkami`");
        inner
    }
}
