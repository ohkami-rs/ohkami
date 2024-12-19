use super::schema::{SchemaRef, Schema, Type::SchemaType};
use serde::Serialize;

pub(crate) const fn is_false(bool: &bool) -> bool {
    !*bool
}

#[derive(Serialize)]
pub(crate) struct Content {
    schema: SchemaRef
}
impl<T: SchemaType> From<Schema<T>> for Content {
    fn from(schema: Schema<T>) -> Self {
        Self { schema: schema.into() }
    }
}
