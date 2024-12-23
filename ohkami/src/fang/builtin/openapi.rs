#![cfg(feature="openapi")]

use crate::openapi::document::Server;
use crate::config::OpenAPIMetadata;
use crate::{Fang, FangProc};
use std::path::PathBuf;


pub struct OpenAPI(OpenAPIMetadata);

impl OpenAPI {
    pub fn json(
        title:   &'static str,
        version: &'static str,
        servers: impl Into<Vec<Server>>
    ) -> Self {
        let file_path = PathBuf::from("openapi.json");
        let servers = Into::<Vec<_>>::into(servers);
        Self(OpenAPIMetadata { title, version, servers, file_path })
    }

    /// Configure the file path to generate.
    /// 
    /// ## default
    /// `openapi.json`
    /// 
    /// ## note
    /// In current cargo workspace, relative paths is treated as
    /// *relative to the workspace root directory*.
    pub fn file(mut self, file_path: impl Into<PathBuf>) -> Self {
        self.0.file_path = file_path.into();
        self
    }
}

impl<Inner: FangProc> Fang<Inner> for OpenAPI {
    type Proc = Inner;
    fn chain(&self, inner: Inner) -> Self::Proc {
        crate::CONFIG.openapi_metadata()
            .set(self.0.clone())
            .ok().expect("[OpenAPI] Unexpected multiple `OpenAPI`s in a `Ohkami`");
        inner
    }
}
