use super::{Response, ResponseHeaders, Content};


macro_rules! status {
    (
        $(
            $code:literal $name:ident : $message:literal,
        )*
    ) => {
        #[derive(PartialEq, Clone, Copy)]
        #[allow(non_camel_case_types)]
        pub enum Status {
            $( $name = $code, )*
        }

        impl Status {
            #[inline(always)]
            pub const fn code(&self) -> u16 {
                *self as _
            }

            #[inline(always)] pub(crate) const fn as_str(&self) -> &'static str {
                match self {
                    $( Self::$name => $message, )*
                }
            }
            #[cfg(feature="__rt_native__")]
            #[inline(always)] pub(crate) const fn line(&self) -> &'static [u8] {
                match self {
                    $( Self::$name => concat!("HTTP/1.1 ", $message, "\r\n").as_bytes(), )*
                }
            }
        }

        impl From<u16> for Status {
            fn from(code: u16) -> Self {
                match code {
                    $($code => Self::$name,)*
                    _ => panic!("Status code out of range: `{code}`")
                }
            }
        }

        #[allow(non_snake_case)]
        impl Response {
            $(
                #[doc = "`"]
                #[doc = $message]
                #[doc = "`"]
                #[doc = "empty Response"]
                #[inline(always)]
                pub fn $name() -> Self {
                    Self {
                        status:  Status::$name,
                        headers: ResponseHeaders::new(),
                        content: Content::None,
                    }
                }
            )*
        }
    };
} status! {
    100 Continue                      : "100 Continue",
    101 SwitchingProtocols            : "101 Switching Protocols",
    102 Processing                    : "102 Processing",
    103 EarlyHints                    : "103 Early Hints",

    200 OK                            : "200 OK",
    201 Created                       : "201 Created",
    202 Accepted                      : "202 Accepted",
    203 NonAuthoritativeInformation   : "203 Non-Authoritative Information",
    204 NoContent                     : "204 No Content",
    205 ResetContent                  : "205 Reset Content",
    206 PartialContent                : "206 Partial Content",
    207 MultiStatus                   : "207 Multi-Status",
    208 AlreadyReported               : "208 Already Reported",
    226 IMUsed                        : "226 IMUsed",

    300 MultipleChoice                : "300 Multiple Choice",
    301 MovedPermanently              : "301 Moved Permanently",
    302 Found                         : "302 Found",
    303 SeeOther                      : "303 See Other",
    304 NotModified                   : "304 Not Modifed",
    307 TemporaryRedirect             : "307 Temporary Redirect",
    308 PermanentRedirect             : "308 Permanent Redirect",

    400 BadRequest                    : "400 Bad Request",
    401 Unauthorized                  : "401 Unauthorized",
    403 Forbidden                     : "403 Forbidden",
    404 NotFound                      : "404 Not Found",
    405 MethodNotAllowed              : "405 Method Not Allowed",
    406 NotAcceptable                 : "406 Not Acceptable",
    407 ProxyAuthenticationRequired   : "407 Proxy Authentication Required",
    408 RequestTimeout                : "408 Request Timeout",
    409 Conflict                      : "409 Conflict",
    410 Gone                          : "410 Gone",
    411 LengthRequired                : "411 Length Required",
    412 PreconditionFailed            : "412 Precondition Failed",
    413 PayloadTooLarge               : "413 Payload Too Large",
    414 URITooLong                    : "414 URI Too Long",
    415 UnsupportedMediaType          : "415 Unsupported Media Type",
    416 RangeNotSatisfiable           : "416 Range Not Satisfiable",
    417 ExceptionFailed               : "417 Exception Failed",
    418 Im_a_teapot                   : "418 I'm a teapot",
    421 MisdirectedRequest            : "421 Misdirected Request",
    422 UnprocessableEntity           : "422 Unprocessable Entity",
    423 Locked                        : "423 Locked",
    424 FailedDependency              : "424 Failed Dependency",
    426 UpgradeRequired               : "426 UpgradeRequired",
    428 PreconditionRequired          : "428 Precondition Required",
    429 TooManyRequest                : "429 Too Many Request",
    431 RequestHeaderFieldsTooLarge   : "431 Request Header Fields Too Large",
    451 UnavailableForLegalReasons    : "451 Unavailable For Legal Reasons",

    500 InternalServerError           : "500 Internal Server Error",
    501 NotImplemented                : "501 Not Implemented",
    502 BadGateway                    : "502 Bad Gateway",
    503 ServiceUnavailable            : "503 Service Unavailable",
    504 GatewayTimeout                : "504 Gateway Timeout",
    505 HTTPVersionNotSupported       : "505 HTTP Version Not Supported",
    506 VariantAlsoNegotiates         : "506 Variant Also Negotiates",
    507 InsufficientStorage           : "507 Unsufficient Storage",
    508 LoopDetected                  : "508 Loop Detected",
    510 NotExtended                   : "510 Not Extended",
    511 NetworkAuthenticationRequired : "511 Network Authentication Required",
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
