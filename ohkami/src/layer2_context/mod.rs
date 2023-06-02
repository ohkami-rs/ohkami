#![allow(non_snake_case)]

use serde::Serialize;
use crate::{
    layer0_lib::{AsStr, Status},
    layer1_req_res::{ResponseHeaders, Response, ErrorResponse, OkResponse},
};


pub struct Context {
    headers: ResponseHeaders,
}

impl Context {
    #[inline(always)] pub(crate) fn new() -> Self {
        Self { headers: ResponseHeaders::new() }
    }
}

impl Context {
    #[inline(always)] pub fn text<Text: AsStr>(&mut self, text: Text) -> Response<Text> {
        self.headers.set("Content-Type", "text/plain");
        Ok(OkResponse::with_body_asstr(
            text,
            Status::OK,
            &self.headers,
        ))
    }
    #[inline(always)] pub fn html<HTML: AsStr>(&mut self, html: HTML) -> Response<HTML> {
        self.headers.set("Content-Type", "text/html");
        Ok(OkResponse::with_body_asstr(
            html,
            Status::OK,
            &self.headers,
        ))
    }
    #[inline(always)] pub fn json<JSON: Serialize>(&mut self, json: JSON) -> Response<JSON> {
        self.headers.set("Content-Type", "application/json");
        Ok(OkResponse::with_body(
            json,
            Status::OK,
            &self.headers,
        ))
    }

    #[inline(always)] pub fn Created<Entity: Serialize>(&mut self, entity: Entity) -> Response<Entity> {
        self.headers.set("Content-Type", "application/json");
        Ok(OkResponse::with_body(
            entity,
            Status::Created,
            &self.headers,
        ))
    }

    #[inline(always)] pub fn NoContent(&mut self) -> Response<()> {
        self.headers.clear("Content-Type");
        Ok(OkResponse::without_body(
            Status::NoContent,
            &self.headers,
        ))
    }
}

macro_rules! impl_error_response {
    ($( $name:ident ),*) => {
        impl Context {
            $(
                #[inline(always)] pub fn $name(&mut self) -> ErrorResponse {
                    self.headers.clear("Content-Type");
                    ErrorResponse::new(Status::$name, &self.headers)
                }
            )*
        }
    };
} impl_error_response!(
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    InternalServerError,
    NotImplemented
);
