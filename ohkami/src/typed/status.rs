use std::borrow::Cow;
use serde::Serialize;
use super::{Payload, PayloadType};
use crate::{IntoResponse, Response, Status};


trait ResponseBody {
    fn into_response_with(self, status: Status) -> Response;
}
const _: () = {
    impl ResponseBody for () {
        fn into_response_with(self, status: Status) -> Response {
            Response::with(status)
        }
    }

    impl<P: Payload + Serialize> ResponseBody for P {
        #[inline]
        fn into_response_with(self, status: Status) -> Response {
            let mut res = Response::with(status);
            {
                macro_rules! content_type {
                    () => {<<Self as Payload>::Type as PayloadType>::CONTENT_TYPE};
                }

                let bytes = match <<Self as Payload>::Type as PayloadType>::bytes(&self) {
                    Ok(bytes) => bytes,
                    Err(e) => return (|| {
                        eprintln!("Failed to serialize {} as {}: {e}", std::any::type_name::<Self>(), content_type!());
                        Response::InternalServerError()
                    })()
                };

                res.headers.set()
                    .ContentType(content_type!())
                    .ContentLength(bytes.len().to_string());

                res.content = Some(std::borrow::Cow::Owned(bytes));
            }
            res
        }
    }
};

macro_rules! generate_statuses_as_types_containing_value {
    ($( $status:ident : $message:literal, )*) => {
        $(
            #[doc = "Type-safe `"]
            #[doc = $message]
            #[doc = "` response with the `ResponseBody`.\n\n---\n"]
            #[doc = "Use `()` (: default) for body to represent an empty-content response of the status: <br>"]
            #[doc = "\
            ```\n\
            # use ohkami::prelude::*;\n\
            # use ohkami::typed::status::OK;\n\
            async fn create_user(name: &str) -> OK {\n\
            \tOK(())\n\
            }\n\
            ```"]
            #[allow(non_camel_case_types)]
            #[allow(private_bounds)]
            pub struct $status<B: ResponseBody = ()>(pub B);

            impl<B: ResponseBody> IntoResponse for $status<B> {
                fn into_response(self) -> Response {
                    self.0.into_response_with(Status::$status)
                }
            }
        )*
    };
} generate_statuses_as_types_containing_value! {
    OK                            : "200 OK",
    Created                       : "201 Created",
    NonAuthoritativeInformation   : "203 Non-Authoritative Information",
    PartialContent                : "206 Partial Content",
    MultiStatus                   : "207 Multi-Status",
    AlreadyReported               : "208 Already Reported",
    IMUsed                        : "226 IMUsed",

    MultipleChoice                : "300 Multiple Choice",

    BadRequest                    : "400 Bad Request",
    Unauthorized                  : "401 Unauthorized",
    Forbidden                     : "403 Forbidden",
    NotFound                      : "404 Not Found",
    MethodNotAllowed              : "405 Method Not Allowed",
    NotAcceptable                 : "406 Not Acceptable",
    ProxyAuthenticationRequired   : "407 Proxy Authentication Required",
    RequestTimeout                : "408 Request Timeout",
    Conflict                      : "409 Conflict",
    LengthRequired                : "411 Length Required",
    PreconditionFailed            : "412 Precondition Failed",
    PayloadTooLarge               : "413 Payload Too Large",
    URITooLong                    : "414 URI Too Long",
    UnsupportedMediaType          : "415 Unsupported Media Type",
    RangeNotSatisfiable           : "416 Range Not Satisfiable",
    ExceptionFailed               : "417 Exception Failed",
    Im_a_teapot                   : "418 I'm a teapot",
    MisdirectedRequest            : "421 Misdirected Request",
    UnprocessableEntity           : "422 Unprocessable Entity",
    Locked                        : "423 Locked",
    FailedDependency              : "424 Failed Dependency",
    UpgradeRequired               : "426 UpgradeRequired",
    PreconditionRequired          : "428 Precondition Required",
    TooManyRequest                : "429 Too Many Request",
    RequestHeaderFieldsTooLarge   : "431 Request Header Fields Too Large",
    UnavailableForLegalReasons    : "451 Unavailable For Legal Reasons",

    InternalServerError           : "500 Internal Server Error",
    NotImplemented                : "501 Not Implemented",
    BadGateway                    : "502 Bad Gateway",
    ServiceUnavailable            : "503 Service Unavailable",
    GatewayTimeout                : "504 Gateway Timeout",
    HTTPVersionNotSupported       : "505 HTTP Version Not Supported",
    VariantAlsoNegotiates         : "506 Variant Also Negotiates",
    InsufficientStorage           : "507 Unsufficient Storage",
    LoopDetected                  : "508 Loop Detected",
    NotExtended                   : "510 Not Extended",
    NetworkAuthenticationRequired : "511 Network Authentication Required",
}

macro_rules! generate_statuses_as_types_with_no_value {
    ($( $status:ident : $message:literal, )*) => {
        $(
            #[doc = "Type-safe `"]
            #[doc = $message]
            #[doc = "` response"]
            pub struct $status;

            impl IntoResponse for $status {
                #[inline] fn into_response(self) -> Response {
                    Status::$status.into_response()
                }
            }
        )*
    };
} generate_statuses_as_types_with_no_value! {
    Continue                      : "100 Continue",
    SwitchingProtocols            : "101 Switching Protocols",
    Processing                    : "102 Processing",
    EarlyHints                    : "103 Early Hints",

    Accepted                      : "202 Accepted",
    NoContent                     : "204 No Content",
    ResetContent                  : "205 Reset Content",

    NotModified                   : "304 Not Modifed",

    Gone                          : "410 Gone",
}

macro_rules! generate_redirects {
    ($( $status:ident / $contructor:ident : $message:literal, )*) => {
        $(
            #[doc = "Type-safe `"]
            #[doc = $message]
            #[doc = "` response using the `location` as `Location` header value"]
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
                    let mut res = Response::with(Status::$status);
                    res.headers.set()
                        .Location(self.location);
                    res
                }
            }
        )*
    };
} generate_redirects! {
    MovedPermanently / to             : "301 Moved Permanently",
    Found / at                        : "302 Found",
    SeeOther / at                     : "303 See Other",
    TemporaryRedirect / to            : "307 Temporary Redirect",
    PermanentRedirect / to            : "308 Permanent Redirect",
}
