use crate::serde::Serialize;
use crate::response::ResponseHeaders;
use crate::{Status, Response};
use std::borrow::Cow;


/// # Response body
/// 
/// Utility trait to be used with `ohkami::typed`.
/// 
/// （In most cases, we recommend using `#[ResponseBody]`）
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
/// use ohkami::typed::{Payload, ResponseBody};
/// use ohkami::typed::status::Created;
/// 
/// enum MyError {
///     Hoge,
/// }
/// impl ohkami::IntoResponse for MyError {
///     fn into_response(self) -> ohkami::Response {
///         ohkami::Response::InternalServerError()
///     }
/// }
/// 
/// #[Payload(JSOND)]
/// struct CreateUserRequest<'c> {
///     name:     &'c str,
///     password: &'c str,
///     bio:      Option<&'c str>,
/// }
/// 
/// #[ResponseBody(JSONS)]
/// struct User {
///     name: String,
///     bio:  Option<String>,
/// }
/// 
/// async fn create_user(
///     req:  CreateUserRequest<'_>,
/// ) -> Result<Created<User>, MyError> {
///     Ok(Created(User {
///         name: req.name.into(),
///         bio:  req.bio.map(String::from),
///     }))
/// }
/// ```
pub trait ResponseBody: Serialize {
    /// Select from `ohkami::typed::bodytype` module
    type Type: BodyType;
    fn into_response_with(self, status: Status) -> Response;
}

pub trait BodyType {}
macro_rules! bodytype {
    ($( $name:ident, )*) => {
        pub mod bodytype {
            $(
                pub struct $name;
                impl super::BodyType for $name {}
            )*
        }
    };
} bodytype! {
    Empty,
    JSON,
    HTML,
    Text,
    Other,
}

impl<RB: ResponseBody> crate::IntoResponse for RB {
    fn into_response(self) -> Response {
        self.into_response_with(Status::OK)
    }
}

impl ResponseBody for () {
    type Type = bodytype::Empty;
    fn into_response_with(self, status: Status) -> Response {
        Response {
            status,
            headers: ResponseHeaders::new(),
            content: None,
        }
    }
}

const _: (/* JSON utility impls */) = {
    impl<RB: ResponseBody<Type = bodytype::JSON>> ResponseBody for Option<RB> {
        type Type = bodytype::JSON;
        fn into_response_with(self, status: Status) -> Response {
            Response::with(status).json(self)
        }
    }

    impl<RB: ResponseBody<Type = bodytype::JSON>> ResponseBody for Vec<RB> {
        type Type = bodytype::JSON;
        #[inline] fn into_response_with(self, status: Status) -> Response {
            Response::with(status).json(self)
        }
    }

    impl<RB: ResponseBody<Type = bodytype::JSON>> ResponseBody for &[RB] {
        type Type = bodytype::JSON;
        fn into_response_with(self, status: Status) -> Response {
            Response::with(status).json(self)
        }
    }

    /// `impl<RB: ResponseBody<Type = bodytype::JSON>, const N: usize> ResponseBody for [RB; N]`
    /// is not available becasue `serde` only provides following 33 `Serialize` impls...
    macro_rules! response_body_of_json_array_of_len {
        ($($len:literal)*) => {
            $(
                impl<RB: ResponseBody<Type = bodytype::JSON>> ResponseBody for [RB; $len] {
                    type Type = bodytype::JSON;
                    #[inline] fn into_response_with(self, status: Status) -> Response {
                        Response::with(status).json(self)
                    }
                }
            )*
        };
    } response_body_of_json_array_of_len! {
        0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17
        18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
    }
};

macro_rules! plain_text_responsebodies {
    ($( $text_type:ty: $self:ident => $content:expr, )*) => {
        $(
            impl ResponseBody for $text_type {
                type Type = bodytype::Text;
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
    fn is_empty_reponsebody<T: ResponseBody<Type = bodytype::Empty>>() {}
    is_empty_reponsebody::<()>();

    fn is_text_reponsebody<T: ResponseBody<Type = bodytype::Text>>() {}
    is_text_reponsebody::<&'static str>();
    is_text_reponsebody::<String>();
    is_text_reponsebody::<&'_ String>();
    is_text_reponsebody::<Cow<'static, str>>();
    is_text_reponsebody::<Cow<'_, str>>();
}
