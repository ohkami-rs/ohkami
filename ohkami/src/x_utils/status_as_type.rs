use std::borrow::Cow;
use serde::Serialize;
use crate::{IntoResponse, Response, layer1_req_res::ResponseHeaders, prelude::Status};


pub trait ResponseBody: Serialize {
    fn into_response_with(self, status: Status) -> Response;
}
macro_rules! plain_text_responsebodies {
    ($( $text_type:ty: $self:ident => $content:expr, )*) => {
        $(
            impl ResponseBody for $text_type {
                #[inline] fn into_response_with(self, status: Status) -> Response {
                    let content = {let $self = self; $content};
            
                    let mut headers = ResponseHeaders::new();
                    headers.set()
                        .ContentType("text/plain; charset=UTF-8")
                        .ContentLength(content.len().to_string());
            
                    Response {
                        status,
                        headers,
                        content: Some(content.into()),
                    }
                }
            }
        )*
    };
} plain_text_responsebodies! {
    &'static str:      s => s.as_bytes(),
    String:            s => s.into_bytes(),
    &'_ String:        s => s.clone().into_bytes(),
    Cow<'static, str>: c => match c {
        Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
        Cow::Owned   (s) => Cow::Owned   (s.into_bytes()),
    },
}

#[cfg(test)]
#[test] fn assert_impls() {
    fn is_reponsebody<T: ResponseBody>() {}

    is_reponsebody::<&'static str>();
    is_reponsebody::<String>();
    is_reponsebody::<&'_ String>();
    is_reponsebody::<Cow<'static, str>>();
    is_reponsebody::<Cow<'_, str>>();
}


macro_rules! generate_statuses_as_types_containing_value {
    ($( $status:ident, )*) => {
        $(
            pub struct $status<B: ResponseBody>(pub B);

            impl<B: ResponseBody> IntoResponse for $status<B> {
                fn into_response(self) -> Response {
                    self.0.into_response_with(Status::$status)
                }
            }
        )*
    };
} generate_statuses_as_types_containing_value! {
    OK,
    Created,

    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    UnprocessableEntity,

    InternalServerError,
}

macro_rules! generate_statuses_as_types_with_no_value {
    ($( $status:ident, )*) => {
        $(
            pub struct $status;

            impl IntoResponse for $status {
                #[inline] fn into_response(self) -> Response {
                    Status::$status.into_response()
                }
            }
        )*
    };
} generate_statuses_as_types_with_no_value! {
    SwitchingProtocols,

    NoContent,

    NotImplemented,
}

macro_rules! generate_redirects {
    ($( $status:ident / $contructor:ident, )*) => {
        $(
            pub struct $status {
                location: Cow<'static, str>,
            }
            impl $status {
                pub fn $contructor(location: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                    Self {
                        location: location.into(),
                    }
                }
            }

            impl IntoResponse for $status {
                #[inline] fn into_response(self) -> Response {
                    let mut res = Response::$status();
                    res.headers.set()
                        .Location(self.location);
                    res
                }
            }
        )*
    };
} generate_redirects! {
    MovedPermanently / to,
    Found / at,
}
