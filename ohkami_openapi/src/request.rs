use super::schema::{SchemaRef, RawSchema};
use super::_util::{Content, Map, is_false};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Parameter {
    #[serde(rename = "in")]
    kind: ParameterKind,

    pub(crate) name: std::borrow::Cow<'static, str>,
    pub(crate) schema: SchemaRef,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,

    required: bool,
    
    #[serde(skip_serializing_if = "is_false")]
    deprecated: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<&'static str>,
    #[serde(skip_serializing_if = "is_false")]
    explode: bool,
}

#[derive(Serialize, Clone)]
enum ParameterKind {
    path,
    query,
    header,
    cookie,
}

impl Parameter {
    pub(crate) fn is_path(&self) -> bool {
        matches!(self.kind, ParameterKind::path)
    }
}

impl Parameter {
    pub fn in_path(schema: impl Into<SchemaRef>) -> Self {
        Self {
            kind: ParameterKind::path,
            name: "".into(), // initialize with empty name (will be assigned later by `Operation::assign_path_param_name`)
            schema: schema.into(),
            required: true,
            description:None, deprecated:false, style:None, explode:false,
        }
    }
    pub fn in_path_optional(schema: impl Into<SchemaRef>) -> Self {
        Self {
            kind: ParameterKind::path,
            name: "".into(), // initialize with empty name (will be assigned later by `Operation::assign_path_param_name`)
            schema: schema.into(),
            required: false,
            description:None, deprecated:false, style:None, explode:false,
        }
    }

    pub fn in_query(name: &'static str, schema: impl Into<SchemaRef>) -> Self {
        Self {
            kind: ParameterKind::query,
            name: name.into(),
            schema: schema.into(),
            required: true,
            description:None, deprecated:false, style:None, explode:false,
        }
    }
    pub fn in_query_optional(name: &'static str, schema: impl Into<SchemaRef>) -> Self {
        Self {
            kind: ParameterKind::query,
            name: name.into(),
            schema: schema.into(),
            required: false,
            description:None, deprecated:false, style:None, explode:false,
        }
    }
    
    pub fn in_header(name: &'static str, schema: impl Into<SchemaRef>) -> Self {
        Self {
            kind: ParameterKind::header,
            name: name.into(),
            schema: schema.into(),
            required: true,
            description:None, deprecated:false, style:None, explode:false,
        }
    }
    pub fn in_header_optional(name: &'static str, schema: impl Into<SchemaRef>) -> Self {
        Self {
            kind: ParameterKind::header,
            name: name.into(),
            schema: schema.into(),
            required: false,
            description:None, deprecated:false, style:None, explode:false,
        }
    }
    
    pub fn in_cookie(name: &'static str, schema: impl Into<SchemaRef>) -> Self {
        Self {
            kind: ParameterKind::cookie,
            name: name.into(),
            schema: schema.into(),
            required: true,
            description:None, deprecated:false, style:None, explode:false,
        }
    }
    pub fn in_cookie_optional(name: &'static str, schema: impl Into<SchemaRef>) -> Self {
        Self {
            kind: ParameterKind::cookie,
            name: name.into(),
            schema: schema.into(),
            required: false,
            description:None, deprecated:false, style:None, explode:false,
        }
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.set_description(description);
        self
    }
    pub(crate) fn set_description(&mut self, description: &'static str) {
        self.description = Some(description);
    }

    pub fn deprecated(mut self) -> Self {
        self.deprecated = true;
        self
    }
}

#[derive(Serialize, Clone)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,

    required: bool,

    content: Map<&'static str, Content>
}

impl RequestBody {
    pub fn of(media_type: &'static str, schema: impl Into<SchemaRef>) -> Self {
        Self {
            description: None,
            required:    true,
            content:     Map::from_iter([(
                media_type,
                Content::from(schema.into())
            )])
        }
    }
    pub fn optional(media_type: &'static str, schema: impl Into<SchemaRef>) -> Self {
        Self {
            description: None,
            required:    false,
            content:     Map::from_iter([(
                media_type,
                Content::from(schema.into())
            )])
        }
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.set_description(description);
        self
    }
    pub(crate) fn set_description(&mut self, description: &'static str) {
        self.description = Some(description);
    }

    pub fn another(mut self, media_type: &'static str, schema: impl Into<SchemaRef>) -> Self {
        self.content.insert(media_type, Content::from(schema.into()));
        self
    }

    pub(crate) fn refize_schemas(&mut self) -> impl Iterator<Item = RawSchema> + '_ {
        self.content.values_mut().map(Content::refize_schema).flatten()
    }
}
