use super::schema::SchemaRef;
use serde::Serialize;

pub(crate) const fn is_false(bool: &bool) -> bool {
    !*bool
}

#[derive(Serialize)]
pub(crate) struct Content {
    pub(crate) schema: SchemaRef
}

