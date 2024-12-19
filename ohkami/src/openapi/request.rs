use super::schema::{Schema, Type::SchemaType};
use super::_util::Content;
use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,

    required: bool,

    content: HashMap<&'static str, Content>
}

impl RequestBody {
    pub fn new<T: SchemaType>(media_type: &'static str, schema: Schema<T>) -> Self {
        Self {
            description: None,
            required:    true,
            content:     HashMap::from_iter([(
                media_type,
                schema.into()
            )])
        }
    }
    pub fn optional<T: SchemaType>(media_type: &'static str, schema: Schema<T>) -> Self {
        Self {
            description: None,
            required:    false,
            content:     HashMap::from_iter([(
                media_type,
                schema.into()
            )])
        }
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn another<T: SchemaType>(mut self, media_type: &'static str, schema: Schema<T>) -> Self {
        self.content.insert(media_type, schema.into());
        self
    }
}
