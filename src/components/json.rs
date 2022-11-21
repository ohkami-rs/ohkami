use serde::{Serialize, Deserialize};
use crate::{
    context::Context,
    response::ResponseFormat
};


#[derive(Debug)]
pub struct JSON(String);
impl<'d> JSON {
    pub fn from_struct<S: Serialize>(value: &S) -> Context<Self> {
        Ok(Self(serde_json::to_string(value)?))
    }
    pub fn to_struct<D: Deserialize<'d>>(&'d self) -> Context<D> {
        Ok(serde_json::from_str(&self.0)?)
    }

    pub(crate) fn from_str_unchecked(str: &str) -> Self {
        Self(str.to_owned())
    }
    pub(crate) fn content_length(&self) -> usize {
        self.0.len()
    }
}

impl ResponseFormat for JSON {
    fn response_format(&self) -> &str {
        self.0.as_str()
    }
}

impl<S: ToString> From<S> for JSON {
    fn from(value: S) -> Self {
        Self(value.to_string())
    }
}