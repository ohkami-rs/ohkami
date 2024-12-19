use super::schema::{Schema, SchemaRef, Type::SchemaType};
use super::_util::{Content, is_false};
use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize)]
pub struct Parameter {
    #[serde(rename = "in")]
    kind: ParameterKind,

    name:  &'static str,
    schema: SchemaRef,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,

    #[serde(skip_serializing_if = "is_false")]
    required: bool,
    #[serde(skip_serializing_if = "is_false")]
    deprecated: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<&'static str>,
    #[serde(skip_serializing_if = "is_false")]
    explode: bool,
}

#[derive(Serialize)]
enum ParameterKind {
    query,
    header,
    path,
    cookie,
}

impl Parameter {
    pub fn in_query<T: SchemaType>(name: &'static str, schema: Schema<T>) -> Self {
        Self {
            kind: ParameterKind::query,
            name, schema:schema.into(),
            required: true,
            description:None, deprecated:false, style:None, explode:false,
        }
    }
    pub fn maybe_in_query<T: SchemaType>(name: &'static str, schema: Schema<T>) -> Self {
        Self {
            kind: ParameterKind::query,
            name, schema:schema.into(),
            required: false,
            description:None, deprecated:false, style:None, explode:false,
        }
    }
    
    pub fn in_header<T: SchemaType>(name: &'static str, schema: Schema<T>) -> Self {
        Self {
            kind: ParameterKind::header,
            name, schema:schema.into(),
            required: true,
            description:None, deprecated:false, style:None, explode:false,
        }
    }
    pub fn maybe_in_header<T: SchemaType>(name: &'static str, schema: Schema<T>) -> Self {
        Self {
            kind: ParameterKind::header,
            name, schema:schema.into(),
            required: false,
            description:None, deprecated:false, style:None, explode:false,
        }
    }
    
    pub fn in_path<T: SchemaType>(name: &'static str, schema: Schema<T>) -> Self {
        Self {
            kind: ParameterKind::path,
            name, schema:schema.into(),
            required: true,
            description:None, deprecated:false, style:None, explode:false,
        }
    }
    pub fn maybe_in_path<T: SchemaType>(name: &'static str, schema: Schema<T>) -> Self {
        Self {
            kind: ParameterKind::path,
            name, schema:schema.into(),
            required: false,
            description:None, deprecated:false, style:None, explode:false,
        }
    }
    
    pub fn in_cookie<T: SchemaType>(name: &'static str, schema: Schema<T>) -> Self {
        Self {
            kind: ParameterKind::cookie,
            name, schema:schema.into(),
            required: true,
            description:None, deprecated:false, style:None, explode:false,
        }
    }
    pub fn maybe_in_cookie<T: SchemaType>(name: &'static str, schema: Schema<T>) -> Self {
        Self {
            kind: ParameterKind::cookie,
            name, schema:schema.into(),
            required: false,
            description:None, deprecated:false, style:None, explode:false,
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

#[derive(Serialize)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,

    required: bool,

    content: HashMap<&'static str, Content>
}

impl RequestBody {
    pub fn new(media_type: &'static str, schema: impl Into<SchemaRef>) -> Self {
        Self {
            description: None,
            required:    true,
            content:     HashMap::from_iter([(
                media_type,
                Content::from(schema.into())
            )])
        }
    }
    pub fn optional(media_type: &'static str, schema: impl Into<SchemaRef>) -> Self {
        Self {
            description: None,
            required:    false,
            content:     HashMap::from_iter([(
                media_type,
                Content::from(schema.into())
            )])
        }
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn another(mut self, media_type: &'static str, schema: impl Into<SchemaRef>) -> Self {
        self.content.insert(media_type, Content::from(schema.into()));
        self
    }
}
