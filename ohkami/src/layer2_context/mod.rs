#![allow(non_snake_case)]

use serde::Serialize;
use crate::{
    layer0_lib::{AsStr, Status, ContentType},
    layer1_req_res::{ResponseHeaders, Response, ErrResponse},
};


pub struct Context {
    pub header: ResponseHeaders,
}

impl Context {
    #[inline(always)] pub(crate) fn new() -> Self {
        Self { header: ResponseHeaders::new() }
    }
}

impl Context {
    #[inline(always)] pub fn text<Text: AsStr>(&self, text: Text) -> Response<Text> {
        Response::ok_with_body_asstr(
            text,
            Status::OK,
            ContentType::Text,
            &self.header,
        )
    }
    #[inline(always)] pub fn html<HTML: AsStr>(&self, html: HTML) -> Response<HTML> {
        Response::ok_with_body_asstr(
            html,
            Status::OK,
            ContentType::HTML,
            &self.header,
        )
    }
    #[inline(always)] pub fn json<JSON: Serialize>(&self, json: JSON) -> Response<JSON> {
        Response::ok_with_body_json(
            json,
            Status::OK,
            &self.header,
        )
    }

    #[inline(always)] pub fn Created<Entity: Serialize>(&self, entity: Entity) -> Response<Entity> {
        Response::ok_with_body_json(
            entity,
            Status::Created,
            &self.header,
        )
    }

    #[inline(always)] pub fn NoContent(&self) -> Response<()> {
        Response::ok_without_body(
            Status::NoContent,
            &self.header,
        )
    }
}

macro_rules! impl_error_response {
    ($( $name:ident ),*) => {
        impl Context {
            $(
                #[inline(always)] pub fn $name(&self) -> ErrResponse {
                    ErrResponse::new(Status::$name, &self.header)
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
        ErrResponse::new(Status::InternalServerError, &self.header)
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
