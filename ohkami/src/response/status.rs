use super::{Response, ResponseHeaders};


macro_rules! status {
    (
        $(
            $name:ident : $message:literal,
        )*
    ) => {
        #[derive(PartialEq, Clone, Copy)]
        #[allow(non_camel_case_types)]
        pub enum Status {
            $( $name, )*
        }

        impl Status {
            #[inline(always)] pub(crate) const fn as_str(&self) -> &'static str {
                match self {
                    $( Self::$name => $message, )*
                }
            }
            #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
            #[inline(always)] pub(crate) const fn as_bytes(&self) -> &'static [u8] {
                self.as_str().as_bytes()
            }
        }

        #[allow(non_snake_case)]
        impl Response {
            $(
                pub fn $name() -> Self {
                    Self {
                        status:  Status::$name,
                        headers: ResponseHeaders::new(),
                        content: None,
                    }
                }
            )*
        }
    };
} status! {
    Continue                      : "100 Continue",
    SwitchingProtocols            : "101 Switching Protocols",
    Processing                    : "102 Processing",
    EarlyHints                    : "103 Early Hints",

    OK                            : "200 OK",
    Created                       : "201 Created",
    Accepted                      : "202 Accepted",
    NonAuthoritativeInformation   : "203 Non-Authoritative Information",
    NoContent                     : "204 No Content",
    ResetContent                  : "205 Reset Content",
    PartialContent                : "206 Partial Content",
    MultiStatus                   : "207 Multi-Status",
    AlreadyReported               : "208 Already Reported",
    IMUsed                        : "226 IMUsed",

    MultipleChoice                : "300 Multiple Choice",
    MovedPermanently              : "301 Moved Permanently",
    Found                         : "302 Found",
    SeeOther                      : "303 See Other",
    NotModified                   : "304 Not Modifed",
    TemporaryRedirect             : "307 Temporary Redirect",
    PermanentRedirect             : "308 Permanent Redirect",

    BadRequest                    : "400 Bad Request",
    Unauthorized                  : "401 Unauthorized",
    Forbidden                     : "403 Forbidden",
    NotFound                      : "404 Not Found",
    MethodNotAllowed              : "405 Method Not Allowed",
    NotAcceptable                 : "406 Not Acceptable",
    ProxyAuthenticationRequired   : "407 Proxy Authentication Required",
    RequestTimeout                : "408 Request Timeout",
    Conflict                      : "409 Conflict",
    Gone                          : "410 Gone",
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

const _: () = {
    impl std::fmt::Debug for Status {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.as_str())
        }
    }
    impl std::fmt::Display for Status {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.as_str())
        }
    }
};
