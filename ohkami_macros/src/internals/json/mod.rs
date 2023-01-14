mod number;
mod object;
mod parse;
mod build;
mod serialize_fmt;

use std::fmt::Debug;

use object::Object;
use number::Number;
use proc_macro2::Ident;

pub(crate) enum JsonStr {
    Num(Number),
    Str(String),
    Bool(bool),
    Array(Vec<JsonStr>),
    Var(Ident),
    Object(Object),
}

impl Clone for JsonStr {
    fn clone(&self) -> Self {
        match self {
            Self::Array(array) => Self::Array(array.clone()),
            Self::Bool(boolean) => Self::Bool(boolean.clone()),
            Self::Num(number) => Self::Num(match number {
                Number::Float(f) => Number::Float(f.clone()),
                Number::Negative(n) => Number::Negative(n.clone()),
                Number::Positive(p) => Number::Positive(p.clone()),
            }),
            Self::Str(string) => Self::Str(string.clone()),
            Self::Var(name) => Self::Var(name.clone()),
            Self::Object(object) => Self::Object(Object(object.0.clone())),
        }
    }
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
