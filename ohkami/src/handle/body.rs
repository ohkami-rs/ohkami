//! # `body` components
//! 
//! This module provides components for typed handling of request and response bodies
//! via the `FromBody` and `IntoBody` traits.
//! 
//! Built-in body formats include:
//! 
//! - `Json`: `application/json` request/response body
//! - `UrlEncoded`: `application/x-www-form-urlencoded` request/response body
//! - `Text`: `text/plain` request/response body
//! - `Html`: `text/html` response body
//! - `Multipart`: `multipart/form-data` request body
//! 
//! ## Example
//! 
//! ```
//! use ohkami::handle::Json;
//! use ohkami::serde::{Serialize, Deserialize};
//! 
//! #[derive(Deserialize)]
//! struct CreateUserRequest<'req> {
//!     name: &'req str,
//! }
//! 
//! #[derive(Serialize)]
//! struct User {
//!     id: u64,
//!     name: String,
//! }
//! 
//! # enum AppError {}
//! # impl ohkami::IntoResponse for AppError {
//! #     fn into_response(self) -> ohkami::Response {
//! #         todo!()
//! #     }
//! # }
//! 
//! async fn create_user(
//!     // Extract `application/json` request body
//!     // and deserialize it into `CreateUserRequest`,
//!     // rejecting when the request doesn't have such body.
//!     Json(body): Json<CreateUserRequest<'_>>,
//! 
//!             // Serialize `User` into a response body
//!             // of `application/json`.
//! ) -> Result<Json<User>, AppError> {
//!     let some_id = 42; // Simulate user creation here...
//! 
//!     Ok(Json(User {
//!         id: some_id,
//!         name: body.name.to_owned(),
//!     }))
//! }
//! ```

use crate::{Request, Response, FromRequest, IntoResponse};
use std::borrow::Cow;

pub trait FromBody<'req>: Sized {
    /// e.g. `application/json` `text/html`
    const MIME_TYPE: &'static str;

    fn from_body(body: &'req [u8]) -> Result<Self, impl std::fmt::Display>;

    #[cfg(feature="openapi")]
    fn openapi_requestbody() -> impl Into<openapi::schema::SchemaRef>;
}
impl<'req, B: FromBody<'req>> FromRequest<'req> for B {
    type Error = Response;
    
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if req.headers.content_type()?.starts_with(B::MIME_TYPE) {
            Some(B::from_body(req.payload()?).map_err(super::reject))
        } else {
            None
        }
    }

    #[cfg(feature="openapi")]
    fn openapi_inbound() -> openapi::Inbound {
        openapi::Inbound::Body(openapi::RequestBody::of(
            B::MIME_TYPE, B::openapi_requestbody()
        ))
    }
}

pub trait IntoBody {
    /// e.g. `text/html; charset=UTF-8`
    const CONTENT_TYPE: &'static str;

    fn into_body(self) -> Result<Cow<'static, [u8]>, impl std::fmt::Display>;

    #[cfg(feature="openapi")]
    fn openapi_responsebody() -> impl Into<openapi::schema::SchemaRef>;
}
impl<B: IntoBody> IntoResponse for B {
    #[inline]
    fn into_response(self) -> Response {
        if const {Self::CONTENT_TYPE.is_empty()} {// removed by optimization if it's not ""
            return Response::OK()
        }

        match self.into_body() {
            Ok(body) => Response::OK().with_payload(Self::CONTENT_TYPE, body),
            Err(_err) => {
                #[cfg(debug_assertions)] {
                    eprintln!("Failed to serialize `{}` as `{}` in `IntoBody`: {_err}",
                        std::any::type_name::<B>(),
                        Self::CONTENT_TYPE
                    )
                }
                Response::InternalServerError()
            }
        }
    }

    #[cfg(feature="openapi")]
    fn openapi_responses() -> openapi::Responses {
        let mut res = openapi::Response::when("OK");
        if Self::CONTENT_TYPE != "" {
            let mime_type = match Self::CONTENT_TYPE.split_once(';') {
                None => Self::CONTENT_TYPE,
                Some((mime_type, _)) => mime_type
            };
            res = res.content(mime_type, Self::openapi_responsebody());
        }
        openapi::Responses::new([(200, res)])
    }
}
impl IntoBody for () {
    const CONTENT_TYPE: &'static str = "";

    #[cold] #[inline(never)]
    fn into_body(self) -> Result<Cow<'static, [u8]>, impl std::fmt::Display> {
        #[allow(unreachable_code)]
        {unreachable!("`into_body` of `()`") as Result<Cow<'static, [u8]>, std::convert::Infallible>}
    }

    #[cfg(feature="openapi")]
    #[cold] #[inline(never)]
    fn openapi_responsebody() -> impl Into<openapi::schema::SchemaRef> {
        #[allow(unreachable_code)]
        {unreachable!("`openapi_responsebody` of `()`") as openapi::schema::SchemaRef}
    }
}
macro_rules! text_response {
    ($( $t:ty: $this:ident => $conv:expr ),*) => {$(
        impl IntoBody for $t {
            const CONTENT_TYPE: &'static str = "text/plain; charset=UTF-8";

            #[inline(always)]
            fn into_body(self) -> Result<Cow<'static, [u8]>, impl std::fmt::Display> {
                let $this = self;
                Ok::<_, std::convert::Infallible>($conv)
            }

            #[cfg(feature="openapi")]
            fn openapi_responsebody() -> impl Into<openapi::schema::SchemaRef> {
                openapi::string()
            }
        }
    )*};
} text_response! {
    &'static str:      s => Cow::Borrowed(s.as_bytes()),
    String:            s => Cow::Owned(s.into_bytes()),
    Cow<'static, str>: s => match s {
        Cow::Owned(s)    => Cow::Owned(s.into_bytes()),
        Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
    }
}

mod json;
pub use json::Json;

mod multipart;
pub use multipart::{Multipart, File};

mod urlencoded;
pub use urlencoded::UrlEncoded;

mod text;
pub use text::Text;

mod html;
pub use html::Html;
