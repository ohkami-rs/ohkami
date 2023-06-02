#![allow(non_snake_case)]

use serde::Serialize;
use crate::{
    layer0_lib::{AsStr, Status, ContentType},
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
        self.headers.setContentType(ContentType::Text);
        Ok(OkResponse::with_body_asstr(
            text,
            Status::OK,
            &mut self.headers,
        ))
    }
    #[inline(always)] pub fn html<HTML: AsStr>(&mut self, html: HTML) -> Response<HTML> {
        self.headers.setContentType(ContentType::HTML);
        Ok(OkResponse::with_body_asstr(
            html,
            Status::OK,
            &mut self.headers,
        ))
    }
    #[inline(always)] pub fn json<JSON: Serialize>(&mut self, json: JSON) -> Response<JSON> {
        self.headers.setContentType(ContentType::JSON);
        Ok(OkResponse::with_body(
            json,
            Status::OK,
            &mut self.headers,
        ))
    }

    #[inline(always)] pub fn Created<Entity: Serialize>(&mut self, entity: Entity) -> Response<Entity> {
        self.headers.setContentType(ContentType::JSON);
        Ok(OkResponse::with_body(
            entity,
            Status::Created,
            &mut self.headers,
        ))
    }

    #[inline(always)] pub fn NoContent(&mut self) -> Response<()> {
        self.headers.clearContentType();
        Ok(OkResponse::without_body(
            Status::NoContent,
            &mut self.headers,
        ))
    }
}

macro_rules! impl_error_response {
    ($( $name:ident ),*) => {
        impl Context {
            $(
                #[inline(always)] pub fn $name(&mut self) -> ErrorResponse {
                    self.headers.clearContentType();
                    ErrorResponse::new(Status::$name, self.headers.others_than_ContentType())
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
