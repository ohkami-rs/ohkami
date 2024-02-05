mod headers;
pub use headers::{Headers as ResponseHeaders, Header as ResponseHeader};

mod into_response;
pub use into_response::IntoResponse;

use std::{
    borrow::Cow,
};
use crate::{
    __rt__::AsyncWriter,
    layer0_lib::Status,
};


/// # HTTP Response
/// 
/// Composed of
/// 
/// - `status`
/// - `headers`
/// - `content`
/// 
/// <br>
/// 
/// ## Usages
/// 
/// ---
/// 
/// *in_fang.rs*
/// ```
/// use ohkami::{Response, Fang, IntoFang};
/// 
/// struct LogResponse;
/// impl IntoFang for LogResponse {
///     fn into_fang(self) -> Fang {
///         Fang(|res: &Response| {
///             println!("{}", res.status);
///         })
///     }
/// }
/// ```
/// 
/// ---
/// 
/// *into_response.rs*
/// ```
/// use ohkami::{Response, IntoResponse, Status};
/// 
/// enum AppError {
///     A(String),
///     B(String),
/// }
/// impl IntoResponse for AppError {
///     fn into_response(self) -> Response {
///         match self {
///             Self::A(msg) => Response::InternalServerError().text(msg),
///             Self::B(msg) => Response::BadRequest().text(msg),
///         }
///     }
/// }
/// 
/// async fn handler(id: usize) -> Result<String, AppError> {
///     if id == 0 {
///         return Err(AppError::B("id must be positive".into()))
///     }
/// 
///     Ok("Hello, Response!".into())
/// }
/// ```
pub struct Response {
    pub status:         Status,
    pub headers:        ResponseHeaders,
    pub(crate) content: Option<Cow<'static, [u8]>>,
} const _: () = {
    impl Response {
        #[inline(always)] pub fn with(status: Status) -> Self {
            Self {
                status,
                headers: ResponseHeaders::new(),
                content: None,
            }
        }
    }

    macro_rules! direct_with_status_method {
        ($( $status:ident, )*) => {
            #[allow(non_snake_case)]
            impl Response {
                $(
                    pub fn $status() -> Self {
                        Self {
                            status:  Status::$status,
                            headers: ResponseHeaders::new(),
                            content: None,
                        }
                    }
                )*
            }
        };
    } direct_with_status_method! {
        Continue,
        SwitchingProtocols,
        Processing,
        EarlyHints,
    
        OK,
        Created,
        Accepted,
        NonAuthoritativeInformation,
        NoContent,
        ResetContent,
        PartialContent,
        MultiStatus,
        AlreadyReported,
        IMUsed,
    
        MultipleChoice,
        MovedPermanently,
        Found,
        SeeOther,
        NotModified,
        TemporaryRedirect,
        PermanentRedirect,
    
        BadRequest,
        Unauthorized,
        Forbidden,
        NotFound,
        MethodNotAllowed,
        NotAcceptable,
        ProxyAuthenticationRequired,
        RequestTimeout,
        Conflict,
        Gone,
        LengthRequired,
        PreconditionFailed,
        PayloadTooLarge,
        URITooLong,
        UnsupportedMediaType,
        RangeNotSatisfiable,
        ExceptionFailed,
        Im_a_teapot,
        MisdirectedRequest,
        UnprocessableEntity,
        Locked,
        FailedDependency,
        UpgradeRequired,
        PreconditionRequired,
        TooManyRequest,
        RequestHeaderFieldsTooLarge,
        UnavailableForLegalReasons,
    
        InternalServerError,
        NotImplemented,
        BadGateway,
        ServiceUnavailable,
        GatewayTimeout,
        HTTPVersionNotSupported,
        VariantAlsoNegotiates,
        InsufficientStorage,
        LoopDetected,
        NotExtended,
        NetworkAuthenticationRequired,
    }
};

impl Response {
    #[inline] pub(crate) fn into_bytes(self) -> Vec<u8> {
        let Self { status, headers, content, .. } = self;

        let mut buf = Vec::from("HTTP/1.1 ");
        buf.extend_from_slice(status.as_bytes());
        buf.extend_from_slice(b"\r\n");
        headers.write_to(&mut buf);
        if let Some(content) = content {
            buf.extend_from_slice(&content);
        }
        
        buf
    }
}

impl Response {
    #[inline(always)] pub(crate) async fn send(self, stream: &mut (impl AsyncWriter + Unpin)) {
        if let Err(e) = stream.write_all(&self.into_bytes()).await {
            panic!("Failed to send response: {e}")
        }
    }
}

impl Response {
    pub fn drop_content(mut self) -> Self {
        self.content = None;
        self.headers.set()
            .ContentType(None)
            .ContentLength(None);
        self
    }

    #[inline] pub fn text<Text: Into<Cow<'static, str>>>(mut self, text: Text) -> Response {
        let body = text.into();

        self.headers.set()
            .ContentType("text/plain; charset=UTF-8")
            .ContentLength(body.len().to_string());
        self.content = Some(match body {
            Cow::Borrowed(s)   => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        });

        self
    }
    #[inline] pub fn html<HTML: Into<Cow<'static, str>>>(mut self, html: HTML) -> Response {
        let body = html.into();

        self.headers.set()
            .ContentType("text/html; charset=UTF-8")
            .ContentLength(body.len().to_string());
        self.content = Some(match body {
            Cow::Borrowed(s)   => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        });

        self
    }
    #[inline] pub fn json<JSON: serde::Serialize>(mut self, json: JSON) -> Response {
        let body = ::serde_json::to_vec(&json).unwrap();

        self.headers.set()
            .ContentType("application/json; charset=UTF-8")
            .ContentLength(body.len().to_string());
        self.content = Some(Cow::Owned(body));

        self
    }
    pub fn json_lit<JSONString: Into<Cow<'static, str>>>(mut self, json_literal: JSONString) -> Response {
        let body = match json_literal.into() {
            Cow::Borrowed(str) => Cow::Borrowed(str.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        };

        self.headers.set()
            .ContentType("application/json; charset=UTF-8")
            .ContentLength(body.len().to_string());
        self.content = Some(body);

        self
    }
}

const _: () = {
    impl std::fmt::Debug for Response {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self.content {
                None => f.debug_struct("Response")
                    .field("status",  &self.status)
                    .field("headers", &self.headers)
                    .finish(),
                Some(cow) => f.debug_struct("Response")
                    .field("status",  &self.status)
                    .field("headers", &self.headers)
                    .field("content", &cow.escape_ascii())
                    .finish(),
            }
        }
    }

    impl PartialEq for Response {
        fn eq(&self, other: &Self) -> bool {
            self.status  == other.status  &&
            self.headers == other.headers &&
            self.content == other.content
        }
    }
};
