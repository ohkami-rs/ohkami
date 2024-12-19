use super::schema::{SchemaRef, Schema, Type::SchemaType};
use super::_util::{Content, is_false};
use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize)]
pub struct Responses {
    responses: HashMap<u16, Response>
}

#[derive(Serialize)]
pub struct Response {
    description: &'static str,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    content: HashMap<&'static str, Content>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    headers: HashMap<&'static str, Header>
}

#[derive(Serialize)]
pub struct Header {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,

    #[serde(skip_serializing_if = "is_false")]
    required: bool,
    #[serde(skip_serializing_if = "is_false")]
    deprecated: bool,

    schema: SchemaRef
}

impl Responses {
    pub fn new<const N: usize>(responses: [(u16, Response); N]) -> Self {
        Self { responses: HashMap::from_iter(responses) }
    }
}

impl Response {
    pub fn of(description: &'static str) -> Self {
        Self {
            description,
            content: HashMap::new(),
            headers: HashMap::new(),
        }
    }

    pub fn content<T: SchemaType>(mut self, media_type: &'static str, schema: Schema<T>) -> Self {
        self.content.insert(media_type, schema.into());
        self
    }

    pub fn header(mut self, name: &'static str, header: impl Into<Header>) -> Self {
        self.headers.insert(name, header.into());
        self
    }
}

impl Header {
    pub fn of<T: SchemaType>(schema: Schema<T>) -> Self {
        Self {
            description: None,
            required:    true,
            deprecated:  false,
            schema: schema.into()
        }
    }
    pub fn optional<T: SchemaType>(schema: Schema<T>) -> Self {
        Self {
            description: None,
            required:    false,
            deprecated:  false,
            schema: schema.into()
        }
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn deprecated(mut self) -> Self {
        self.deprecated = true;
        self
    }
}
impl<T: SchemaType> From<Schema<T>> for Header {
    fn from(schema: Schema<T>) -> Self {
        Self::of(schema)
    }
}
