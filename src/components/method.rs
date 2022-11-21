use std::fmt::Display;

use crate::{
    context::Context,
    response::Response
};


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
Clone, Copy)]
pub(crate) enum Method {
    GET,
    POST,
    PATCH,
    DELETE,
}

impl Method {
    pub(crate) fn parse(string: &str) -> Context<Self> {
        match string {
            "GET"    => Ok(Self::GET),
            "POST"   => Ok(Self::POST),
            "PATCH"  => Ok(Self::PATCH),
            "DELETE" => Ok(Self::DELETE),
            _ => Err(Response::BadRequest(format!("invalid request method: `{string}`"))),
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