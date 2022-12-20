use std::fmt::Display;
use crate::{
    response::Response, result::Result
};


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
Clone, Copy)]
pub enum Method {
    GET,
    POST,
    PATCH,
    DELETE,
}

impl Method {
    pub(crate) fn parse(string: &str) -> Result<Self> {
        match string {
            "GET"    => Ok(Self::GET),
            "POST"   => Ok(Self::POST),
            "PATCH"  => Ok(Self::PATCH),
            "DELETE" => Ok(Self::DELETE),
            _ => Err(Response::BadRequest(format!("invalid request method: `{string}`"))),
        }
    }
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::GET    => 3,
            Self::POST   => 4,
            Self::PATCH  => 5,
            Self::DELETE => 6,
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::GET    => "GET",
            Self::POST   => "POST",
            Self::PATCH  => "PATCH",
            Self::DELETE => "DELETE",
        })
    }
}