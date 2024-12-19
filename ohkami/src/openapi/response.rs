use super::schema::SchemaRef;
use super::_util::{Content, is_false};
use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize)]
pub struct Responses {
    responses: HashMap<u16, Response>
}

#[derive(Serialize)]
struct Response {
    description: &'static str,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    content: HashMap<&'static str, Content>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    headers: HashMap<&'static str, Header>
}

#[derive(Serialize)]
struct Header {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,

    #[serde(skip_serializing_if = "is_false")]
    required: bool,
    #[serde(skip_serializing_if = "is_false")]
    deprecated: bool,

    schema: SchemaRef
}

impl Responses {
    
}
