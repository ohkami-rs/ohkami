use serde::Serialize;
use crate::{
    layer0_lib::{AsStr, Status},
    layer1_req_res::{ResponseHeaders, Response, response_with_body, response_with_body_asstr, response_without_body},
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
    #[inline(always)] pub fn append(&mut self, key: &str, value: impl AsStr) {
        self.headers.append(key, value)
    }
    #[inline(always)] pub fn set(&mut self, key: &str, value: impl AsStr) {
        self.headers.set(key, value)
    }
    #[inline(always)] pub fn clear(&mut self, key: &str) {
        self.headers.clear(key)
    }
}

impl Context {
    #[inline(always)] pub fn text<Text: AsStr>(&mut self, text: Text) -> Response<Text> {
        self.set("Content-Type", "text/plain");
        response_with_body_asstr(
            text,
            Status::OK,
            &self.headers,
        )
    }
    #[inline(always)] pub fn html<HTML: AsStr>(&mut self, html: HTML) -> Response<HTML> {
        self.set("Content-Type", "text/html");
        response_with_body_asstr(
            html,
            Status::OK,
            &self.headers,
        )
    }
    #[inline(always)] pub fn json<JSON: Serialize>(&mut self, json: JSON) -> Response<JSON> {
        self.set("Content-Type", "application/json");
        response_with_body(
            json,
            Status::OK,
            &self.headers,
        )
    }

    #[inline(always)] pub fn Created<Entity: Serialize>(&mut self, entity: Entity) -> Response<Entity> {
        self.set("Content-Type", "application/json");
        response_with_body(
            entity,
            Status::Created,
            &self.headers,
        )
    }

    #[inline(always)] pub fn NoContent(&mut self) -> Response<()> {
        self.clear("Content-Type");
        response_without_body(
            Status::NoContent,
            &self.headers,
        )
    }
}

impl Context {
    #[inline(always)] pub fn BadRequest<T: Serialize, Content: AsStr>(&mut self, content: Content) -> Response<T> {
        self.headers.append_if_not_has("Content-Type", "text/plain");
    }
}

