use super::{paths::Paths, Operations};
use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize)]
pub struct Document {
    openapi: &'static str,
    info:    Info,
    servers: Vec<Server>,
    paths:   Paths
}

#[derive(Serialize)]
struct Info {
    title:   &'static str,
    version: &'static str,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,
}

#[derive(Serialize)]
pub struct Server {
    url: &'static str,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<Box<HashMap<&'static str, ServerVariable>>>
}
#[derive(Serialize)]
struct ServerVariable {
    default: &'static str,

    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    enumerates: Vec<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,
}

impl Server {
    pub fn at(url: &'static str) -> Self {
        Self { url, description:None, variables:None }
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn var<const N: usize>(
        mut self,
        name: &'static str,
        default: &'static str,
        candidates: [&'static str; N]
    ) -> Self {
        if self.variables.is_none() {
            self.variables = Some(Box::new(HashMap::new()))
        }
        self.variables.as_mut().unwrap().insert(
            name,
            ServerVariable {
                default,
                enumerates:  candidates.into(),
                description: None
            });
        self
    }
}

impl Document {
    pub fn new<const N: usize>(
        title:   &'static str,
        version: &'static str,
        servers: [Server; N]
    ) -> Self {
        Self {
            openapi: "3",
            info:    Info { title, version, description:None },
            servers: servers.into(),
            paths:   Paths::new()
        }
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.info.description = Some(description);
        self
    }

    pub fn path(mut self, path: &'static str, operations: Operations) -> Self {
        self.paths = self.paths.at(path, operations);
        self
    }
}
