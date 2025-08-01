use crate::{Request, Response, FromRequest, IntoResponse, util::ErrorMessage};
use super::bound::{self};
use std::borrow::Cow;

#[cfg(feature="openapi")]
use crate::openapi;

/// # Path parameters
/// 
/// ```ignore
/// Path<(T1, T2, ...)> // some params as tuple
/// Path<T> // single param
/// ```
/// 
/// Parse path parameters of a request into specified type(s)
/// that impl [`FromParam`], in order of their appearance in the path.
/// 
/// When `openapi` feature is activated, each param type is
/// additionally required to impl `ohkami::openapi::Schema`.
/// 
/// ### example
/// 
/// ```no_run
/// use ohkami::{Ohkami, Route};
/// use ohkami::format::{Json, Path};
/// 
/// # enum MyError {}
/// # impl ohkami::IntoResponse for MyError {
/// #     fn into_response(self) -> ohkami::Response {todo!()}
/// # }
/// # #[derive(ohkami::serde::Serialize)]
/// # struct Team {}
/// # #[derive(ohkami::serde::Serialize)]
/// # struct User {}
/// 
/// async fn get_team_info(
///     Path(id): Path<&str>,
/// ) -> Result<Json<Team>, MyError> {
///     todo!()
/// }
/// 
/// async fn get_user_info(
///    Path((team_id, user_id)): Path<(&str, &str)>,
/// ) -> Result<Json<User>, MyError> {
///    todo!()
/// }
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         "/teams/:id"
///             .GET(get_team_info),
///         "/teams/:team_id/users/:user_id"
///             .GET(get_user_info),
///     )).howl("localhost:5050").await
/// }
/// ```
pub struct Path<T>(pub T);

/// "Retrieved from a path param", an element of [`Path<_>`].
/// 
/// When `openapi` feature is activated, `ohkami::openapi::Schema`
/// is required to be impl.
/// 
/// ### required
/// - `type Errpr`
/// - `fn from_param`
pub trait FromParam<'p>: bound::Schema + Sized {
    /// If this extraction never fails, `std::convert::Infallible` is recomended.
    type Error: IntoResponse;

    /// `param` is already percent-decoded：
    /// 
    /// - `Cow::Borrowed(&'p str)` if not encoded in request
    /// - `Cow::Owned(String)` if encoded and ohkami has decoded
    fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error>;

    #[inline(always)]
    fn from_raw_param(raw_param: &'p [u8]) -> Result<Self, Response> {
        Self::from_param(
            ohkami_lib::percent_decode_utf8(raw_param)
                .map_err(|_e| {
                    #[cfg(debug_assertions)] crate::WARNING!(
                        "Failed to decode percent encoded param `{}`: {_e}",
                        raw_param.escape_ascii()
                    );
                    Response::InternalServerError()
                })?
        ).map_err(IntoResponse::into_response)
    }

    #[cfg(feature="openapi")]
    fn openapi_param() -> openapi::Parameter {
        openapi::Parameter::in_path(Self::schema())
    }
}
const _: () = {
    impl<'p> FromParam<'p> for String {
        type Error = std::convert::Infallible;

        #[inline(always)]
        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            Ok(match param {
                Cow::Owned(s)    => s,
                Cow::Borrowed(s) => s.into()
            })
        }
    }
    impl<'p> FromParam<'p> for Cow<'p, str> {
        type Error = std::convert::Infallible;

        #[inline(always)]
        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            Ok(param)
        }
    }
    impl<'p> FromParam<'p> for &'p str {
        type Error = ErrorMessage;

        #[inline(always)]
        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            match param {
                Cow::Borrowed(s) => Ok(s),
                Cow::Owned(_) => Err({
                    #[cold] #[inline(never)]
                    fn unexpected(param: &str) -> ErrorMessage {                        
                        crate::WARNING!("\
                            `&str` can't handle percent encoded parameters. \
                            Use `Cow<'_, str>` or `String` to handle them. \
                        ");
                        ErrorMessage(format!(    
                            "Unexpected path params `{param}`: percent encoded"
                        ))
                    } unexpected(&param)
                }),
            }
        }
    }

    macro_rules! unsigned_integers {
        ($( $unsigned_int:ty ),*) => {
            $(
                impl<'p> FromParam<'p> for $unsigned_int {
                    type Error = ErrorMessage;

                    fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
                        ::byte_reader::Reader::new(param.as_bytes())
                            .read_uint()
                            .map(|i| Self::try_from(i).ok())
                            .flatten()
                            .ok_or_else(|| ErrorMessage(format!("Unexpected path param")))
                    }
                }
            )*
        };
    }
    unsigned_integers! { u8, u16, u32, u64, usize }

    macro_rules! signed_integers {
        ($( $signed_int:ty ),*) => {
            $(
                impl<'p> FromParam<'p> for $signed_int {
                    type Error = ErrorMessage;

                    fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
                        ::byte_reader::Reader::new(param.as_bytes())
                            .read_int()
                            .map(|i| Self::try_from(i).ok())
                            .flatten()
                            .ok_or_else(|| ErrorMessage(format!("Unexpected path param")))
                    }
                }
            )*
        };
    }
    signed_integers! { i8, i16, i32, i64, isize }
};

impl<'req, P: FromParam<'req>> FromRequest<'req> for Path<P> {
    type Error = Response;

    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        // SAFETY:
        // 
        // 1. This extraction is executed only in a handler (created by `IntoHandler::into_handler`)
        // 2. Before `IntoHandler::into_handler` is called, `router::base::Router::finalize`
        //    has already checked that the number of params expected by the handler
        //    matches the number of params in the request path.
        let p = unsafe { req.path.assume_one_param() };
        Some(P::from_raw_param(p).map(Path))
    }

    #[cfg(feature="openapi")]
    fn openapi_inbound() -> openapi::Inbound {
        openapi::Inbound::Params(vec![P::openapi_param()])
    }

    fn n_params() -> usize {
        1
    }
}

impl<'req, P1: FromParam<'req>> FromRequest<'req> for Path<(P1,)> {
    type Error = Response;

    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        // SAFETY: same as above
        let p = unsafe { req.path.assume_one_param() };
        Some(P1::from_raw_param(p).map(|p1| Path((p1,))))
    }

    #[cfg(feature="openapi")]
    fn openapi_inbound() -> openapi::Inbound {
        openapi::Inbound::Params(vec![P1::openapi_param()])
    }

    fn n_params() -> usize {
        1
    }
}

impl<'req, P1: FromParam<'req>, P2: FromParam<'req>> FromRequest<'req> for Path<(P1, P2)> {
    type Error = Response;

    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        // SAFETY: same as above
        let (p1, p2) = unsafe { req.path.assume_two_params() };
        Some(match (P1::from_raw_param(p1), P2::from_raw_param(p2)) {
            (Ok(p1), Ok(p2)) => Ok(Path((p1, p2))),
            (Err(e), _) | (_, Err(e)) => Err(e),
        })
    }

    #[cfg(feature="openapi")]
    fn openapi_inbound() -> openapi::Inbound {
        openapi::Inbound::Params(vec![
            P1::openapi_param(),
            P2::openapi_param(),
        ])
    }

    fn n_params() -> usize {
        2
    }
}
