#![allow(non_snake_case)]

use crate::{Response, Status};

#[cfg(feature="openapi")]
use crate::openapi;

use std::borrow::Cow;


/// A trait implemented to be a returned value of a handler
/// 
/// <br>
/// 
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// 
/// struct MyResponse {
///     message: String,
/// }
/// impl IntoResponse for MyResponse {
///     fn into_response(self) -> Response {
///         Response::OK().with_text(self.message)
///     }
/// }
/// 
/// async fn handler() -> MyResponse {
///     MyResponse {
///         message: String::from("Hello!")
///     }
/// }
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new(
///         "/".GET(handler)
///     ).howl("localhost:5050").await
/// }
/// ```
pub trait IntoResponse {
    fn into_response(self) -> Response;

    #[cfg(feature="openapi")]
    fn openapi_responses() -> openapi::Responses;
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

impl IntoResponse for Response {
    #[inline] fn into_response(self) -> Response {
        self
    }

    #[cfg(feature="openapi")]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new([])
    }
}

impl IntoResponse for Status {
    #[inline(always)] fn into_response(self) -> Response {
        Response::new(self)
    }

    #[cfg(feature="openapi")]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new([])
    }
}

impl<T:IntoResponse, E:IntoResponse> IntoResponse for Result<T, E> {
    #[inline(always)] fn into_response(self) -> Response {
        match self {
            Ok(ok) => ok.into_response(),
            Err(e) => e.into_response(),
        }
    }

    #[cfg(feature="openapi")]
    fn openapi_responses() -> openapi::Responses {
        let mut res = E::openapi_responses();
        res.merge(T::openapi_responses());
        res
    }
}

impl IntoResponse for std::convert::Infallible {
    #[cold] #[inline(never)]
    fn into_response(self) -> Response {
        unsafe {std::hint::unreachable_unchecked()}
    }

    #[cfg(feature="openapi")]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new([])
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

#[cfg(feature="rt_worker")]
impl IntoResponse for worker::Error {
    fn into_response(self) -> Response {
        Response::InternalServerError().with_text(self.to_string())
    }

    #[cfg(feature="openapi")]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new([(500, openapi::Response::when("Internal error in worker"))])
    }
}
