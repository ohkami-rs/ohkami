use super::{paths::{Paths, Operations}, schema::RawSchema, security::SecurityScheme, _util::Map};
use serde::Serialize;

#[derive(Serialize)]
pub struct Document {
    openapi:    &'static str,
    info:       Info,
    servers:    Vec<Server>,
    paths:      Paths,

    #[serde(skip_serializing_if = "Components::is_empty")]
    components: Components,
}

#[derive(Serialize)]
struct Info {
    title:   &'static str,
    version: &'static str,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,
}

#[derive(Serialize, Clone)]
pub struct Server {
    url: &'static str,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<Box<Map<&'static str, ServerVariable>>>
}
#[derive(Serialize, Clone)]
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
            self.variables = Some(Box::new(Map::new()))
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

#[derive(Serialize, Clone)]
pub struct Components {
    #[serde(skip_serializing_if = "Map::is_empty")]
    schemas:         Map<&'static str, RawSchema>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    securitySchemes: Map<&'static str, SecurityScheme>,
}
impl Components {
    fn is_empty(&self) -> bool {
        self.schemas.is_empty() && self.securitySchemes.is_empty()
    }
}

impl Document {
    pub fn new(
        title:   &'static str,
        version: &'static str,
        servers: impl Into<Vec<Server>>
    ) -> Self {
        Self {
            openapi:    "3.1.0",
            info:       Info { title, version, description:None },
            servers:    servers.into(),
            paths:      Paths::new(),
            components: Components { schemas:Map::new(), securitySchemes:Map::new() }
        }
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.info.description = Some(description);
        self
    }
    pub fn path(mut self, path: impl Into<String>, operations: Operations) -> Self {
        self.paths = self.paths.at(path, operations);
        self
    }
    
    #[doc(hidden)]
    pub fn register_schema_component(&mut self, schema: impl Into<RawSchema>) {
        let schema: RawSchema = schema.into();
        if let Some(name) = schema.__name__ {
            match self.components.schemas.get(&name) {
                Some(it) if *it == schema => return,
                Some(_) => panic!("[OpenAPI] `components.schemas`: contradict registrations of multiple `{name}`s"),
                None => self.components.schemas.insert(name, schema),
            }
        }
    }
    #[doc(hidden)]
    pub fn register_securityScheme_component(&mut self, securityScheme: SecurityScheme) {
        match self.components.securitySchemes.get(&securityScheme.__name__) {
            Some(it) if *it == securityScheme => return,
            Some(_) => panic!("[OpenAPI] `components.securitySchemes`: contradict registrations of multiple `{}`s", securityScheme.__name__),
            None => self.components.securitySchemes.insert(securityScheme.__name__, securityScheme),
        }
    }
}
