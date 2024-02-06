mod status;
pub use status::Status;

mod headers;
pub use headers::{Headers as ResponseHeaders, Header as ResponseHeader};

mod into_response;
pub use into_response::IntoResponse;

use std::{
    borrow::Cow,
};
use crate::__rt__::AsyncWriter;


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
///         Fang::back(|res: &Response| {
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
    /// Headers of this response
    /// 
    /// - `.{HeaderName}()` to get the value
    /// - `.set().{HeaderName}(〜)` to set the value
    /// - `.set().{HeaderName}(append(〜))` to append the value
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
    pub fn drop_content(&mut self) {
        self.content = None;
        self.headers.set()
            .ContentType(None)
            .ContentLength(None);
    }

    #[inline] pub fn text<Text: Into<Cow<'static, str>>>(mut self, text: Text) -> Self {
        self.set_text(text);
        self
    }
    #[inline] pub fn set_text<Text: Into<Cow<'static, str>>>(&mut self, text: Text) {
        let body = text.into();

        self.headers.set()
            .ContentType("text/plain; charset=UTF-8")
            .ContentLength(body.len().to_string());
        self.content = Some(match body {
            Cow::Borrowed(s)   => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        });
    }

    #[inline] pub fn html<HTML: Into<Cow<'static, str>>>(mut self, html: HTML) -> Self {
        self.set_html(html);
        self
    }
    #[inline] pub fn set_html<HTML: Into<Cow<'static, str>>>(&mut self, html: HTML) {
        let body = html.into();

        self.headers.set()
            .ContentType("text/html; charset=UTF-8")
            .ContentLength(body.len().to_string());
        self.content = Some(match body {
            Cow::Borrowed(s)   => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        });
    }

    #[inline] pub fn json<JSON: serde::Serialize>(mut self, json: JSON) -> Self {
        self.set_json(json);
        self
    }
    #[inline] pub fn set_json<JSON: serde::Serialize>(&mut self, json: JSON) {
        let body = ::serde_json::to_vec(&json).unwrap();

        self.headers.set()
            .ContentType("application/json; charset=UTF-8")
            .ContentLength(body.len().to_string());
        self.content = Some(Cow::Owned(body));
    }

    pub fn json_str<JSONString: Into<Cow<'static, str>>>(mut self, json_str: JSONString) -> Self {
        self.set_json_str(json_str);
        self
    }
    pub fn set_json_str<JSONString: Into<Cow<'static, str>>>(&mut self, json_str: JSONString) {
        let body = match json_str.into() {
            Cow::Borrowed(str) => Cow::Borrowed(str.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        };

        self.headers.set()
            .ContentType("application/json; charset=UTF-8")
            .ContentLength(body.len().to_string());
        self.content = Some(body);
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
