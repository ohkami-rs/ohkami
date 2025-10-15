use super::bound::{self};
use crate::{FromRequest, IntoResponse, Request, Response, util::ErrorMessage};
use std::borrow::Cow;

#[cfg(feature = "openapi")]
use crate::openapi;

/// # Query parameters
///
/// Deserialize query parameters in a request into an instance of
/// schema type `T: Deserialize<'_>`.
///
/// When `openapi` feature is activated, schema bound additionally
/// requires `openapi::Schema`.
///
/// ### example
///
/// ```
/// # enum MyError {}
/// use ohkami::claw::{Json, Query};
/// use ohkami::serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct ListUsersMeta<'req> {
///     prefix: Option<&'req str>,
///     min_age: Option<u8>,
///     max_age: Option<u8>,
///     limit: Option<usize>,
/// }
/// # #[derive(ohkami::serde::Serialize)]
/// # struct User {}
///
/// async fn list_users(
///     Query(meta): Query<ListUsersMeta<'_>>,
/// ) -> Result<Json<Vec<User>>, MyError> {
///     todo!()
/// }
/// ```
///
/// ```shell
/// $ curl 'http://localhost:5050/users?prefix=ohkami&limit=100'
/// ```
///
/// ### note
///
/// When a request doesn't have query parameters, `Option<Query<T>>` in a handler
/// tries to deserialize an *empty query string*, not skip deserializing
/// with returning `None`.
/// This may be unexpected behavior and just *`Query<T>` with `Option<_>` fields*
/// is recommended to express *optional query params*.
pub struct Query<T: bound::Schema>(pub T);

impl<'req, T: bound::Incoming<'req>> FromRequest<'req> for Query<T> {
    type Error = Response;

    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        req.query.parse().map_err(super::reject).map(Query).into()
    }

    #[cfg(feature = "openapi")]
    fn openapi_inbound() -> openapi::Inbound {
        let Some(schema) = T::schema().into().into_inline() else {
            return openapi::Inbound::None;
        };
        openapi::Inbound::Params(
            schema
                .into_properties()
                .into_iter()
                .map(|(name, schema, required)| {
                    if required {
                        openapi::Parameter::in_query(name, schema)
                    } else {
                        openapi::Parameter::in_query_optional(name, schema)
                    }
                })
                .collect(),
        )
    }
}

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
/// use ohkami::claw::{Json, Path};
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
///     )).run("localhost:5050").await
/// }
/// ```
pub struct Path<T>(pub T);

/// "Retrieved from a path param", an element of [`Path<_>`].
///
/// When `openapi` feature is activated, `ohkami::openapi::Schema`
/// is required to be impl.
///
/// ### required
/// - `type Error: IntoResponse`
/// - `fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error>`
///
/// ### default impls
/// - `String` ... own a param as it is, percent decoded
/// - `&str` ... borrow a param, only accepting non-encoded one, rejecting if percent encoded
/// - `Cow<'_, str>` ... own decoded if percent encoded, or borrow if not encoded
/// - `uuid::Uuid` ... parsed from a percent-decoded param
/// - primitive integers ... parsed from a percent-decoded param
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
        Self::from_param(ohkami_lib::percent_decode_utf8(raw_param).map_err(|_e| {
            #[cfg(debug_assertions)]
            crate::WARNING!(
                "Failed to decode percent encoded param `{}`: {_e}",
                raw_param.escape_ascii()
            );
            Response::InternalServerError()
        })?)
        .map_err(IntoResponse::into_response)
    }

    #[cfg(feature = "openapi")]
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
                Cow::Owned(s) => s,
                Cow::Borrowed(s) => s.into(),
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
                    #[cold]
                    #[inline(never)]
                    fn unexpected(param: &str) -> ErrorMessage {
                        crate::WARNING!(
                            "\
                            `&str` can't handle percent encoded parameters. \
                            Use `Cow<'_, str>` or `String` to handle them. \
                        "
                        );
                        ErrorMessage(format!("Unexpected path params `{param}`: percent encoded"))
                    }
                    unexpected(&param)
                }),
            }
        }
    }

    macro_rules! unsigned_integers {
        ($( $unsigned_int:ty ),*) => {
            $(
                impl<'p> FromParam<'p> for $unsigned_int {
                    type Error = Response;

                    fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
                        ::byte_reader::Reader::new(param.as_bytes())
                            .read_uint()
                            .map(|i| Self::try_from(i).ok())
                            .flatten()
                            .ok_or_else(|| {
                                #[cfg(debug_assertions)] {
                                    crate::WARNING!(
                                        "Failed to parse `{}` from path param `{}`",
                                        stringify!($unsigned_int), param
                                    );
                                }
                                Response::BadRequest().with_text("invalid path")
                            })
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
                    type Error = Response;

                    fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
                        ::byte_reader::Reader::new(param.as_bytes())
                            .read_int()
                            .map(|i| Self::try_from(i).ok())
                            .flatten()
                            .ok_or_else(|| {
                                #[cfg(debug_assertions)] {
                                    crate::WARNING!(
                                        "Failed to parse `{}` from path param `{}`",
                                        stringify!($signed_int), param
                                    );
                                }
                                Response::BadRequest().with_text("invalid path")
                            })
                    }
                }
            )*
        };
    }
    signed_integers! { i8, i16, i32, i64, isize }

    impl<'p> FromParam<'p> for uuid::Uuid {
        type Error = Response;

        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            uuid::Uuid::try_parse(&param).map_err(|_| {
                #[cfg(debug_assertions)]
                {
                    crate::WARNING!("Failed to parse UUID from path param `{param}`",);
                }
                Response::BadRequest().with_text("invalid path")
            })
        }
    }
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

    #[cfg(feature = "openapi")]
    fn openapi_inbound() -> openapi::Inbound {
        openapi::Inbound::Params(vec![P::openapi_param()])
    }

    fn n_pathparams() -> usize {
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

    #[cfg(feature = "openapi")]
    fn openapi_inbound() -> openapi::Inbound {
        openapi::Inbound::Params(vec![P1::openapi_param()])
    }

    fn n_pathparams() -> usize {
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

    #[cfg(feature = "openapi")]
    fn openapi_inbound() -> openapi::Inbound {
        openapi::Inbound::Params(vec![P1::openapi_param(), P2::openapi_param()])
    }

    fn n_pathparams() -> usize {
        2
    }
}
