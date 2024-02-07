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
/// use ohkami::typed::{Created, Payload, ResponseBody};
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
    fn into_response_with(self, status: Status) -> Response;
}
impl ResponseBody for () {
    fn into_response_with(self, status: Status) -> Response {
        Response {
            status,
            headers: ResponseHeaders::new(),
            content: None,
        }
    }
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
