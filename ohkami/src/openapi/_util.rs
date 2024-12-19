use super::schema::SchemaRef;
use serde::Serialize;

pub(crate) const fn is_false(bool: &bool) -> bool {
    !*bool
}

#[derive(Serialize)]
pub(crate) struct Content {
    schema: SchemaRef
}
impl<T: Into<SchemaRef>> From<T> for Content {
    fn from(schema: T) -> Self {
        Self { schema: schema.into() }
    }
}
