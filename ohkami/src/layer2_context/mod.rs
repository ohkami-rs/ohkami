#![allow(non_snake_case)]

use serde::Serialize;
use crate::{
    layer0_lib::{AsStr, Status, ContentType},
    layer1_req_res::{ResponseHeaders, Response, ErrResponse},
};


pub struct Context {
    pub headers: ResponseHeaders,
}

impl Context {
    #[inline(always)] pub(crate) fn new() -> Self {
        Self { headers: ResponseHeaders::new() }
    }
}

impl Context {
    #[inline(always)] pub fn Text<Text: AsStr>(&self, text: Text) -> Response<Text> {
        Response::ok_with_body_asstr(
            text,
            Status::OK,
            ContentType::Text,
            &self.headers,
        )
    }
    #[inline(always)] pub fn HTML<HTML: AsStr>(&self, html: HTML) -> Response<HTML> {
        Response::ok_with_body_asstr(
            html,
            Status::OK,
            ContentType::HTML,
            &self.headers,
        )
    }
    #[inline(always)] pub fn JSON<JSON: Serialize>(&self, json: JSON) -> Response<JSON> {
        Response::ok_with_body_json(
            json,
            Status::OK,
            &self.headers,
        )
    }

    #[inline(always)] pub fn Created<Entity: Serialize>(&self, entity: Entity) -> Response<Entity> {
        Response::ok_with_body_json(
            entity,
            Status::Created,
            &self.headers,
        )
    }

    #[inline(always)] pub fn NoContent(&self) -> Response<()> {
        Response::ok_without_body(
            Status::NoContent,
            &self.headers,
        )
    }
}

impl Context {
    #[inline(always)] pub fn Redirect(&self, location: impl AsStr) -> Response {
        Response::redirect(
            location,
            Status::Found,
            &self.headers,
        )
    }
    #[inline(always)] pub fn RedirectPermanently(&self, location: impl AsStr) -> Response {
        Response::redirect(
            location,
            Status::MovedPermanently,
            &self.headers,
        )
    }
}

macro_rules! impl_error_response {
    ($( $name:ident ),*) => {
        impl Context {
            $(
                #[inline(always)] pub fn $name(&self) -> ErrResponse {
                    ErrResponse::new(Status::$name, &self.headers)
                }
            )*
        }
    };
} impl_error_response!(
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    // InternalServerError, // too long ↓
    NotImplemented
); impl Context {
    #[inline(always)] pub fn InternalError(&self) -> ErrResponse {
        ErrResponse::new(Status::InternalServerError, &self.headers)
    }
}




#[cfg(test)] #[allow(unused)] mod __ {
    use crate::{Context, Response};
    use serde::Serialize;

    #[derive(Serialize)]
    struct User {
        id: usize,
        name: String,
    }
    impl User {
        async fn create(name: impl ToString) -> Result<Self, std::io::Error> {
            Ok(Self {
                id: 42,
                name: name.to_string(),
            })
        }
    }

    async fn create_user(c: Context) -> Response<User> {
        let new_user = User::create("John").await
            .map_err(|e| c.InternalError())?;

        c.Created(new_user)
    }
}
