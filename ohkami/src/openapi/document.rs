use super::{paths::Paths, Operations, Response, schema::RawSchema, security::RawSecurityScheme, _util::Map};
use serde::Serialize;

#[derive(Serialize)]
pub struct Document {
    openapi:    &'static str,
    info:       Info,
    servers:    Vec<Server>,
    paths:      Paths,
    components: Components,
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
    variables: Option<Box<Map<&'static str, ServerVariable>>>
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
pub(crate) struct Components {
    schemas:         Map<&'static str, RawSchema>,
    responses:       Map<&'static str, Response>,
    securitySchemes: Map<&'static str, RawSecurityScheme>,
}
impl Components {
    pub(crate) fn new() -> Self {
        Components {
            schemas:         Map::new(),
            responses:       Map::new(),
            securitySchemes: Map::new(),
        }        
    }

    pub fn register_schema(&mut self, name: &'static str, schema: impl Into<RawSchema>) {
        let schema: RawSchema = schema.into();
        match self.schemas.get(&name) {
            Some(it) if *it == schema => return,
            Some(_) => panic!("[OpenAPI] `components.schemas`: contradict registrations of multiple `{name}`s"),
            None => self.schemas.insert(name, schema),
        }
    }
    pub fn register_response(&mut self, name: &'static str, response: Response) {
        match self.responses.get(&name) {
            Some(it) if *it == response => return,
            Some(_) => panic!("[OpenAPI] `components.responses`: contradict registrations of multiple `{name}`s"),
            None => self.responses.insert(name, response),
        }
    }
    pub fn register_securityScheme(&mut self, name: &'static str, securityScheme: impl Into<RawSecurityScheme>) {
        let securityScheme: RawSecurityScheme = securityScheme.into();
        match self.securitySchemes.get(&name) {
            Some(it) if *it == securityScheme => return,
            Some(_) => panic!("[OpenAPI] `components.securitySchemes`: contradict registrations of multiple `{name}`s"),
            None => self.securitySchemes.insert(name, securityScheme),
        }
    }
}

impl Document {
    pub fn new<const N: usize>(
        title:   &'static str,
        version: &'static str,
        servers: [Server; N]
    ) -> Self {
        Self {
            openapi:    "3.0.0",
            info:       Info { title, version, description:None },
            servers:    servers.into(),
            paths:      Paths::new(),
            components: Components::new()
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
    pub(crate) fn components(mut self, components: Components) -> Self {
        self.components = components;
       self 
    }
}
