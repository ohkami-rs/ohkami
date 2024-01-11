use std::borrow::Cow;
use serde::Serialize;
use crate::{IntoResponse, Response, layer1_req_res::ResponseHeaders, prelude::Status};


pub trait ResponseBody: Serialize {
    fn into_response_with(self, status: Status) -> Response;
} const _: () = {
    impl<C: Serialize + Into<Cow<'static, str>>> ResponseBody for C {
        #[inline] fn into_response_with(self, status: Status) -> Response {
            let content = match self.into() {
                Cow::Borrowed(str) => Cow::Borrowed(str.as_bytes()),
                Cow::Owned(string) => Cow::Owned(string.into_bytes())
            };
    
            let mut headers = ResponseHeaders::new();
            headers.set()
                .ContentType("text/plain; charset=UTF-8")
                .ContentLength(content.len().to_string());
    
            Response {
                status,
                headers,
                content: Some(content),
            }
        }
    }
};

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

    MovedPermanently,
    Found,

    NotImplemented,
}