#![cfg(feature="openapi")]

use crate::{config, openapi, Fang, FangProc};
use std::path::PathBuf;


pub struct OpenAPI(config::OpenAPIMetadata);

impl OpenAPI {
    /// Register metadata for generating OpenAPI document (JSON).
    /// 
    /// ## note
    /// For now, YAML version is not supported!
    /// 
    /// ## example
    /// 
    /// ```
    /// use ohkami::prelude::*;
    /// use ohkami::fang::OpenAPI;
    /// use ohkami::openapi::Server;
    /// 
    /// fn my_ohkami() -> Ohkami {
    ///     Ohkami::with((
    ///         OpenAPI::json("Sample API", "0.1.9", [
    ///             Server::at("http://api.example.com/v1")
    ///                 .description("Main (production) server"),
    ///             Server::at("http://staging-api.example.com")
    ///                 .description("Internal staging server for testing")
    ///         ]),
    ///     ), (
    ///         "/hello"
    ///             .GET(|| async {"Hello, OpenAPI!"}),
    ///     ))
    /// }
    /// ```
    pub fn json(
        title:   &'static str,
        version: &'static str,
        servers: impl Into<Vec<openapi::document::Server>>
    ) -> Self {
        let servers   = Into::<Vec<_>>::into(servers);
        let file_path = PathBuf::from("openapi.json");
        Self(config::OpenAPIMetadata { title, version, servers, file_path })
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
