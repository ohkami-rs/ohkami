mod number;
mod object;
mod parse;
mod build;

use std::fmt::Debug;

use object::Map;
use number::Number;

pub(super) enum JsonStr {
    Num(Number),
    Str(String),
    Bool(bool),
    Array(Vec<JsonStr>),
    Var(String),
    Object(Map),
}

impl Debug for JsonStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Num(number) => write!(f, "{number:?}"),
            Self::Str(string) => write!(f, "{string:?}"),
            Self::Bool(boolean) => write!(f, "{boolean:?}"),
            Self::Array(array) => write!(f, "{array:?}"),
            Self::Var(variable) => write!(f, "{variable:?}"),
            Self::Object(object) => write!(f, "{object:?}"),
        }
    }
}
