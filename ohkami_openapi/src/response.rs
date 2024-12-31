use super::schema::{SchemaRef, Schema, RawSchema, Type::SchemaType};
use super::_util::{Content, Map, is_false};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Responses(Map<String, Response>);

#[derive(Serialize, Clone, PartialEq)]
pub struct Response {
    description: &'static str,

    #[serde(skip_serializing_if = "Map::is_empty")]
    content: Map<&'static str, Content>,

    #[serde(skip_serializing_if = "Map::is_empty")]
    headers: Map<&'static str, ResponseHeader>
}

#[derive(Serialize, Clone, PartialEq)]
pub struct ResponseHeader {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,

    required: bool,
    
    #[serde(skip_serializing_if = "is_false")]
    deprecated: bool,

    schema: SchemaRef
}

impl Responses {
    pub fn new(code: u16, response: Response) -> Self {
        Self(Map::from_iter([(code.to_string(), response)]))
    }

    pub fn enumerated<const N: usize>(responses: [(u16, Response); N]) -> Self {
        Self(Map::from_iter(responses.map(|(code, res)| (code.to_string(), res))))
    }

    pub fn or(mut self, code: u16, response: Response) -> Self {
        self.0.insert(code.to_string(), response);
        self
    }

    pub fn or_default(mut self, default_response: Response) -> Self {
        self.0.insert("default".to_string(), default_response);
        self
    }

    pub fn merge(&mut self, another: Self) {
        for (code, res) in another.0 {
            self.0.insert(code, res);
        }
    }

    pub(crate) fn refize_schemas(&mut self) -> impl Iterator<Item = RawSchema> + '_ {
        self.0.values_mut().map(Response::refize_schemas).flatten()
    }
}

impl Response {
    pub fn when(description: &'static str) -> Self {
        Self {
            description,
            content: Map::new(),
            headers: Map::new(),
        }
    }

    pub fn content(mut self, media_type: &'static str, schema: impl Into<SchemaRef>) -> Self {
        if media_type != "" {
            self.content.insert(media_type, Content::from(schema.into()));
        }
        self
    }

    pub fn header(mut self, name: &'static str, header: impl Into<ResponseHeader>) -> Self {
        self.headers.insert(name, header.into());
        self
    }

    pub(self) fn refize_schemas(&mut self) -> impl Iterator<Item = RawSchema> {
        let mut refizeds = vec![];
        for content in self.content.values_mut() {
            if let Some(refized) = content.refize_schema() {
                refizeds.push(refized);
            }
        }
        refizeds.into_iter()
    }
}

impl ResponseHeader {
    pub fn of(schema: impl Into<SchemaRef>) -> Self {
        Self {
            description: None,
            required:    true,
            deprecated:  false,
            schema: schema.into()
        }
    }
    pub fn optional(schema: impl Into<SchemaRef>) -> Self {
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
impl<T: SchemaType> From<Schema<T>> for ResponseHeader {
    fn from(schema: Schema<T>) -> Self {
        Self::of(schema)
    }
}
