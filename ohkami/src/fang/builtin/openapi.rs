#![cfg(feature="openapi")]

use std::path::PathBuf;


pub struct OpenAPI {
    pub(crate) file_path: PathBuf,
}

impl OpenAPI {
    pub fn json(file_path: impl Into<PathBuf>) -> Self {
        Self { file_path: file_path.into() }
    }
}

/*
    `OpenAPI` doesn't impl `Fang`, but available in fangs tuple of `Ohkami::with`
    due to special treatment in `Fangs` trait
*/
