use super::{
    _util::Map,
    paths::{Operations, Paths},
    schema::RawSchema,
    security::SecurityScheme,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct Document {
    openapi: &'static str,
    info: Info,
    servers: Vec<Server>,
    paths: Paths,

    #[serde(skip_serializing_if = "Components::is_empty")]
    components: Components,
}

#[derive(Serialize)]
struct Info {
    title: String,
    version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct Server {
    url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<Box<Map<&'static str, ServerVariable>>>,
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
    pub fn at(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            description: None,
            variables: None,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn var<const N: usize>(
        mut self,
        name: &'static str,
        default: &'static str,
        candidates: [&'static str; N],
    ) -> Self {
        if self.variables.is_none() {
            self.variables = Some(Box::new(Map::new()))
        }
        self.variables.as_mut().unwrap().insert(
            name,
            ServerVariable {
                default,
                enumerates: candidates.into(),
                description: None,
            },
        );
        self
    }
}

#[derive(Serialize, Clone)]
pub struct Components {
    #[serde(skip_serializing_if = "Map::is_empty")]
    schemas: Map<&'static str, RawSchema>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    #[serde(rename = "securitySchemes")]
    security_schemes: Map<&'static str, SecurityScheme>,
}
impl Components {
    fn is_empty(&self) -> bool {
        self.schemas.is_empty() && self.security_schemes.is_empty()
    }
}

impl Document {
    pub fn new(
        title: impl Into<String>,
        version: impl Into<String>,
        servers: impl Into<Vec<Server>>,
    ) -> Self {
        Self {
            openapi: "3.1.0",
            info: Info {
                title: title.into(),
                version: version.into(),
                description: None,
            },
            servers: servers.into(),
            paths: Paths::new(),
            components: Components {
                schemas: Map::new(),
                security_schemes: Map::new(),
            },
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.info.description = Some(description.into());
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
                Some(it) if *it == schema => (),
                Some(_) => panic!(
                    "[OpenAPI] `components.schemas`: contradict registrations of multiple `{name}`s"
                ),
                None => self.components.schemas.insert(name, schema),
            }
        }
    }
    #[doc(hidden)]
    pub fn register_securityScheme_component(&mut self, securityScheme: SecurityScheme) {
        match self
            .components
            .security_schemes
            .get(&securityScheme.__name__)
        {
            Some(it) if *it == securityScheme => (),
            Some(_) => panic!(
                "[OpenAPI] `components.security_schemes`: contradict registrations of multiple `{}`s",
                securityScheme.__name__
            ),
            None => self
                .components
                .security_schemes
                .insert(securityScheme.__name__, securityScheme),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_server_at_with_str_or_string() {
        let _: Server = Server::at("http://localhost:3000");
        let _: Server = Server::at(String::from("https://localhost") + ":3000");
    }
    
    #[test]
    fn test_document_new_with_str_or_string() {
        let _: Document = Document::new(
            "title",
            "version",
            &[Server::at("http://localhost:3000")],
        );
        let _: Document = Document::new(
            format!("title"),
            format!("version"),
            &[Server::at(format!("https://localhost:3000"))],
        );
    }
}
