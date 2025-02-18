use crate::{IntoBody, IntoResponse, Response, Status};
use crate::response::{ResponseHeaders, SetHeaders, Content};

#[cfg(feature="openapi")]
use crate::openapi;

macro_rules! generate_statuses_as_types_containing_value {
    ($( $status:ident : $message:literal, )*) => {
        $(
            #[doc = "Type-safe `"]
            #[doc = $message]
            #[doc = "` response.<br>"]
            #[doc = "Use `()` (default) to represent an empty content."]

            #[allow(non_camel_case_types)]
            #[allow(private_bounds)]
            pub struct $status<B: IntoBody = ()>(pub B);

            impl<B: IntoBody> IntoResponse for $status<B> {
                #[inline]
                fn into_response(self) -> Response {
                    let mut res = self.0.into_response();
                    res.status = Status::$status;
                    res
                }

                #[cfg(feature="openapi")]
                fn openapi_responses() -> openapi::Responses {
                    let (code, message) = $message.split_once(' ').unwrap();
                    let mut res = openapi::Response::when(message);
                    if B::CONTENT_TYPE != "" {
                        res = res.content(B::CONTENT_TYPE, B::openapi_responsebody())
                    }
                    openapi::Responses::new([(
                        code.parse().unwrap(),
                        res
                    )])
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
                #[inline]
                fn into_response(self) -> Response {
                    Status::$status.into_response()
                }

                #[cfg(feature="openapi")]
                fn openapi_responses() -> crate::openapi::Responses {
                    let (code, message) = $message.split_once(' ').unwrap();
                    openapi::Responses::new([(
                        code.parse().unwrap(),
                        openapi::Response::when(message)
                    )])
                }
            }
        )*
    };
} generate_statuses_as_types_with_no_value! {
    Continue           : "100 Continue",
    SwitchingProtocols : "101 Switching Protocols",
    Processing         : "102 Processing",
    EarlyHints         : "103 Early Hints",

    Accepted           : "202 Accepted",
    NoContent          : "204 No Content",
    ResetContent       : "205 Reset Content",

    NotModified        : "304 Not Modifed",

    Gone               : "410 Gone",
}

macro_rules! generate_redirects {
    ($( $status:ident / $contructor:ident : $message:literal, )*) => {
        $(
            #[doc = "Type-safe `"]
            #[doc = $message]
            #[doc = "` response using the `location` as `Location` header value"]
            pub struct $status {
                headers: ResponseHeaders,
            }
            impl $status {
                pub fn $contructor(location: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                    let mut headers = ResponseHeaders::new();
                    headers.set().Location(location.into());
                    Self { headers }
                }

                pub fn with_headers(mut self, set: impl FnOnce(SetHeaders)->SetHeaders) -> Self {
                    set(self.headers.set());
                    self
                }
            }

            impl IntoResponse for $status {
                #[inline]
                fn into_response(self) -> Response {
                    Response {
                        status: Status::$status,
                        headers: self.headers,
                        content: Content::None,
                    }
                }

                #[cfg(feature="openapi")]
                fn openapi_responses() -> crate::openapi::Responses {
                    let (code, message) = $message.split_once(' ').unwrap();
                    openapi::Responses::new([(
                        code.parse().unwrap(),
                        openapi::Response::when(message)
                    )])
                }
            }
        )*
    };
} generate_redirects! {
    MovedPermanently / to  : "301 Moved Permanently",
    Found / at             : "302 Found",
    SeeOther / at          : "303 See Other",
    TemporaryRedirect / to : "307 Temporary Redirect",
    PermanentRedirect / to : "308 Permanent Redirect",
}
