use super::schema::{SchemaRef, Schema, RawSchema, Type::SchemaType};
use super::_util::{Content, Map, is_false};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Responses(Map<Status, Response>);

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Status {
    Code(u16),
    Default,
}
impl Serialize for Status {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (match self {
            Self::Code(code) => std::borrow::Cow::Owned(code.to_string()),
            Self::Default    => std::borrow::Cow::Borrowed("default")
        }).serialize(serializer)
    }
}
impl From<u16> for Status {
    fn from(code: u16) -> Self {Self::Code(code)}
}
impl From<&str> for Status {
    fn from(s: &str) -> Self {
        match s {
            "default" => Self::Default,
            _ => Self::Code(s.parse().expect("invalid status code"))
        }
    }
}

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
    pub fn new<const N: usize>(code_responses: [(u16, Response); N]) -> Self {
        Self(Map::from_iter(code_responses.map(|(code, res)| (Status::Code(code), res))))
    }

    pub fn or(mut self, code: u16, response: Response) -> Self {
        self.0.insert(Status::Code(code), response);
        self
    }

    pub fn or_default(mut self, default_response: Response) -> Self {
        self.0.insert(Status::Default, default_response);
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

    pub(crate) fn override_response_description(&mut self, status: impl Into<Status>, new_description: &'static str) {
        let status = status.into();
        if let Some(response) = self.0.get_mut(&status) {
            response.description = new_description;
        } else {
            self.0.insert(status.into(), Response::when(new_description));
        }
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
            for refized in content.refize_schema() {
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
