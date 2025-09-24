use super::content::IntoContent;
use crate::response::{Content, ResponseHeaders, SetHeaders};
use crate::{IntoResponse, Response, Status};

#[cfg(feature = "openapi")]
use crate::openapi;

macro_rules! generate_statuses_as_types_containing_value {
    ($( $status:ident : $message:literal, )*) => {
        $(
            /// Generate`
            #[doc = $message]
            /// ` response type with the `content: C`.
            ///
            /// Use `()` to represent an empty content.
            ///
            /// This is an alias of `typed::{TheStatus}::new(content)`.
            #[allow(non_snake_case)]
            pub fn $status<C: IntoContent>(content: C) -> $status<C> {
                $status::<C>::new(content)
            }

            #[doc = "Typed `"]
            #[doc = $message]
            #[doc = "` response.<br>"]
            #[doc = "Use `()` ( default of `C` ) to represent an empty content."]
            #[allow(private_bounds)]
            pub struct $status<C: IntoContent = ()> {
                content: C,
                headers: ResponseHeaders,
            }

            impl<C: IntoContent> $status<C> {
                pub fn new(content: C) -> Self {
                    Self { content, headers: ResponseHeaders::new() }
                }

                pub fn with_headers(mut self, set: impl FnOnce(SetHeaders)->SetHeaders) -> Self {
                    set(self.headers.set());
                    self
                }
            }

            impl<C: IntoContent> IntoResponse for $status<C> {
                #[inline]
                fn into_response(self) -> Response {
                    if const {C::CONTENT_TYPE.is_empty()} {// will be removed by optimization if it's not
                        return Response::OK();
                    }

                    let content = match self.content.into_content() {
                        Ok(content) => content,
                        Err(e) => {
                            crate::ERROR!("<{} as IntoContent>::into_content() failed: {e}", std::any::type_name::<C>());
                            return Response::InternalServerError();
                        }
                    };

                    let mut headers = self.headers;
                    headers.set()
                        .content_type(C::CONTENT_TYPE)
                        .content_length(ohkami_lib::num::itoa(content.len()));

                    Response {
                        status: Status::$status,
                        headers,
                        content: Content::Payload(content.into())
                    }
                }

                #[cfg(feature="openapi")]
                fn openapi_responses() -> openapi::Responses {
                    let (code, message) = $message.split_once(' ').unwrap();
                    let mut res = openapi::Response::when(message);
                    if C::CONTENT_TYPE != "" {
                        res = res.content(C::CONTENT_TYPE, C::openapi_responsebody())
                    }
                    openapi::Responses::new([(
                        code.parse().unwrap(),
                        res
                    )])
                }
            }
        )*
    };
}
generate_statuses_as_types_containing_value! {
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
    ImATeapot                     : "418 I'm a teapot",
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
            #[doc = "Typed `"]
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
}
generate_statuses_as_types_with_no_value! {
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
            #[doc = "Typed `"]
            #[doc = $message]
            #[doc = "` response using the `location` as `Location` header value"]
            pub struct $status {
                headers: ResponseHeaders,
            }
            impl $status {
                pub fn $contructor(location: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                    let mut headers = ResponseHeaders::new();
                    headers.set().location(location.into());
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
}
generate_redirects! {
    MovedPermanently / to  : "301 Moved Permanently",
    Found / at             : "302 Found",
    SeeOther / at          : "303 See Other",
    TemporaryRedirect / to : "307 Temporary Redirect",
    PermanentRedirect / to : "308 Permanent Redirect",
}

#[cfg(not(feature = "rt_worker"/* panics due to `cannot call wasm-bindgen imported functions on non-wasm targets` */))]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn typed_success_status() {
        assert_eq!(
            Created("Hello, world!").into_response(),
            Response::Created().with_text("Hello, world!")
        );

        assert_eq!(
            Created("Hello, world!")
                .with_headers(|h| h.server("ohkami").vary("origin"))
                .into_response(),
            Response::Created()
                .with_text("Hello, world!")
                .with_headers(|h| h.server("ohkami").vary("origin"))
        );
    }

    #[test]
    fn typed_redirect() {
        assert_eq!(
            Found::at("https://example.com").into_response(),
            Response::Found().with_headers(|h| h.location("https://example.com"))
        );

        assert_eq!(
            Found::at("https://example.com")
                .with_headers(|h| h.server("ohkami"))
                .into_response(),
            Response::Found().with_headers(|h| h.location("https://example.com").server("ohkami"))
        );
    }
}
