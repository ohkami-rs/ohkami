mod status;
pub use status::Status;

mod headers;
#[cfg(feature = "DEBUG")]
pub use headers::Header as ResponseHeader;
pub use headers::{Headers as ResponseHeaders, SetHeaders};

mod content;
pub use content::Content;

mod into_response;
pub use into_response::IntoResponse;

#[cfg(test)]
mod _test;
#[cfg(test)]
mod _test_headers;

use ohkami_lib::{CowSlice, Slice};
use std::borrow::Cow;

#[cfg(feature = "__rt_native__")]
use crate::__rt__::AsyncWrite;
#[cfg(feature = "sse")]
use crate::{
    sse,
    util::{Stream, StreamExt},
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
/// ## Usage
///
/// *in_fang.rs*
/// ```no_run
/// use ohkami::prelude::*;
///
/// #[derive(Clone)]
/// struct SetHeaders;
/// impl FangAction for SetHeaders {
///     async fn back<'a>(&'a self, res: &'a mut Response) {
///         res.headers.set()
///             .server("ohkami")
///             .vary("Origin");
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((SetHeaders,
///         "/".GET(|| async {"Hello, ohkami!"})
///     )).run("localhost:5050").await
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
///             Self::A(msg) => Response::InternalServerError().with_text(msg),
///             Self::B(msg) => Response::BadRequest().with_text(msg),
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
    /// HTTP status of this response
    pub status: Status,

    /// Headers of this response
    ///
    /// - `.{name}()`, `.get("{name}")` to get value
    /// - `.set().{name}({action})`, `.set().x("{name}", {action})` to mutate values
    ///
    /// ---
    ///
    /// `{action}`:
    /// - just `{value}` to insert
    /// - `None` to remove
    /// - `header::append({value})` to append
    ///
    /// `{value}`:
    /// - `String`
    /// - `&'static str`
    /// - `Cow<'static, str>`
    /// - `Some(Cow<'static, str>)`
    pub headers: ResponseHeaders,

    pub(crate) content: Content,
}

impl Response {
    #[inline(always)]
    pub fn new(status: Status) -> Self {
        Self {
            status,
            headers: ResponseHeaders::new(),
            content: Content::None,
        }
    }

    /// complete HTTP spec
    ///
    /// should be called, like, just after router's handling
    #[cfg(feature = "__rt__")]
    #[inline(always)]
    pub(crate) fn complete(&mut self) {
        match (&self.content, &self.status) {
            (_, Status::NoContent) => {
                if self.headers.content_length().is_some() {
                    self.headers.set().content_length(None);
                }
                if !/* not */matches!(self.content, Content::None) {
                    self.content = Content::None;
                }
            }
            #[cfg(feature = "sse")]
            (Content::Stream(_), _) => {
                if self.headers.content_length().is_some() {
                    self.headers.set().content_length(None);
                }
            }
            #[cfg(not(feature="rt_lambda"/* currently */))]
            #[cfg(all(feature = "ws", feature = "__rt__"))]
            (Content::WebSocket(_), _) => {
                if self.headers.content_length().is_some() {
                    self.headers.set().content_length(None);
                }
            }
            _ => (/* let it go by user's responsibility */),
        }
    }
}

impl Response {
    #[inline]
    pub fn with_headers(mut self, f: impl FnOnce(SetHeaders) -> SetHeaders) -> Self {
        f(self.headers.set());
        self
    }

    pub fn drop_content(&mut self) -> Content {
        let old_content = self.content.take();
        self.headers.set().content_type(None).content_length(None);
        old_content
    }
    pub fn without_content(mut self) -> Self {
        let _ = self.drop_content();
        self
    }

    #[inline]
    pub fn set_payload(
        &mut self,
        content_type: &'static str,
        content: impl Into<Cow<'static, [u8]>>,
    ) {
        let content: Cow<'static, [u8]> = content.into();
        self.headers
            .set()
            .content_type(content_type)
            .content_length(ohkami_lib::num::itoa(content.len()));
        self.content = Content::Payload(content.into());
    }
    #[inline]
    pub fn with_payload(
        mut self,
        content_type: &'static str,
        content: impl Into<Cow<'static, [u8]>>,
    ) -> Self {
        self.set_payload(content_type, content);
        self
    }
    pub fn payload(&self) -> Option<&[u8]> {
        self.content.as_bytes()
    }

    #[inline]
    pub fn set_text<Text: Into<Cow<'static, str>>>(&mut self, text: Text) {
        let body: Cow<'static, str> = text.into();

        self.headers
            .set()
            .content_type("text/plain; charset=UTF-8")
            .content_length(ohkami_lib::num::itoa(body.len()));
        self.content = Content::Payload(match body {
            Cow::Borrowed(str) => CowSlice::Ref(Slice::from_bytes(str.as_bytes())),
            Cow::Owned(string) => CowSlice::Own(string.into_bytes().into()),
        });
    }
    #[inline(always)]
    pub fn with_text<Text: Into<Cow<'static, str>>>(mut self, text: Text) -> Self {
        self.set_text(text);
        self
    }

    pub fn set_html<HTML: Into<Cow<'static, str>>>(&mut self, html: HTML) {
        let body: Cow<'static, str> = html.into();

        self.headers
            .set()
            .content_type("text/html; charset=UTF-8")
            .content_length(ohkami_lib::num::itoa(body.len()));
        self.content = Content::Payload(match body {
            Cow::Borrowed(str) => CowSlice::Ref(Slice::from_bytes(str.as_bytes())),
            Cow::Owned(string) => CowSlice::Own(string.into_bytes().into()),
        });
    }
    pub fn with_html<HTML: Into<Cow<'static, str>>>(mut self, html: HTML) -> Self {
        self.set_html(html);
        self
    }

    #[inline(always)]
    pub fn set_json<JSON: serde::Serialize>(&mut self, json: JSON) {
        let body = ::serde_json::to_vec(&json).unwrap();
        self.headers
            .set()
            .content_type("application/json")
            .content_length(ohkami_lib::num::itoa(body.len()));
        self.content = Content::Payload(body.into());
    }
    #[inline(always)]
    pub fn with_json<JSON: serde::Serialize>(mut self, json: JSON) -> Self {
        self.set_json(json);
        self
    }

    /// ## SAFETY
    ///
    /// argument `json_lit` must be **valid JSON**
    pub unsafe fn set_json_lit<JSONLiteral: Into<Cow<'static, str>>>(
        &mut self,
        json_lit: JSONLiteral,
    ) {
        let body = match json_lit.into() {
            Cow::Borrowed(str) => Cow::Borrowed(str.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        };

        self.headers
            .set()
            .content_type("application/json")
            .content_length(ohkami_lib::num::itoa(body.len()));
        self.content = Content::Payload(body.into());
    }
    /// ## SAFETY
    ///
    /// argument `json_lit` must be **valid JSON**
    pub unsafe fn with_json_lit<JSONLiteral: Into<Cow<'static, str>>>(
        mut self,
        json_lit: JSONLiteral,
    ) -> Self {
        unsafe {
            self.set_json_lit(json_lit);
        }
        self
    }
}

#[cfg(feature = "sse")]
#[cfg_attr(docsrs, doc(cfg(feature = "sse")))]
impl Response {
    pub fn with_stream<T: sse::Data>(
        mut self,
        stream: impl Stream<Item = T> + Unpin + Send + 'static,
    ) -> Self {
        self.set_stream(stream);
        self
    }

    pub fn set_stream<T: sse::Data>(
        &mut self,
        stream: impl Stream<Item = T> + Unpin + Send + 'static,
    ) {
        self.set_stream_raw(Box::pin(stream.map(sse::Data::encode)));
    }

    pub fn set_stream_raw(&mut self, stream: std::pin::Pin<Box<dyn Stream<Item = String> + Send>>) {
        self.headers
            .set()
            .content_length(None)
            .content_type("text/event-stream")
            .cache_control("no-cache, must-revalidate")
            .transfer_encoding("chunked");
        self.content = Content::Stream(stream);
    }
}

#[cfg(feature = "__rt_native__")]
pub(super) enum Upgrade {
    None,

    #[cfg(feature = "ws")]
    WebSocket(mews::WebSocket<crate::session::Connection>),
}
#[cfg(feature = "__rt_native__")]
impl Upgrade {
    #[inline(always)]
    pub(super) const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}
#[cfg(feature = "__rt_native__")]
impl Response {
    #[cfg_attr(not(feature = "sse"), inline)]
    pub(crate) async fn send(
        self,
        conn: &mut (impl AsyncWrite + Unpin),
    ) -> std::io::Result<Upgrade> {
        match self.content {
            Content::None => {
                let mut buf =
                    Vec::<u8>::with_capacity(self.status.line().len() + self.headers.size);
                unsafe {
                    crate::push_unchecked!(buf <- self.status.line());
                    self.headers.write_unchecked_to(&mut buf);
                }
                conn.write_all(&buf).await?;
                conn.flush().await?;

                Ok(Upgrade::None)
            }

            Content::Payload(bytes) => {
                let mut buf = Vec::<u8>::with_capacity(
                    self.status.line().len() + self.headers.size + bytes.len(),
                );
                unsafe {
                    crate::push_unchecked!(buf <- self.status.line());
                    self.headers.write_unchecked_to(&mut buf);
                    crate::push_unchecked!(buf <- bytes);
                }
                conn.write_all(&buf).await?;
                conn.flush().await?;

                Ok(Upgrade::None)
            }

            #[cfg(feature = "sse")]
            Content::Stream(mut stream) => {
                let mut buf =
                    Vec::<u8>::with_capacity(self.status.line().len() + self.headers.size);
                unsafe {
                    crate::push_unchecked!(buf <- self.status.line());
                    self.headers.write_unchecked_to(&mut buf);
                }
                conn.write_all(&buf).await?;
                conn.flush().await?;

                while let Some(chunk) = stream.next().await {
                    let mut message = Vec::with_capacity(
                        /* capacity for a single line */
                        "data: ".len() + chunk.len() + "\n\n".len(),
                    );
                    for line in chunk.split('\n') {
                        message.extend_from_slice(b"data: ");
                        message.extend_from_slice(line.as_bytes());
                        message.push(b'\n');
                    }
                    message.push(b'\n');

                    let size_hex_bytes = ohkami_lib::num::hexized_bytes(message.len());

                    let mut chunk = Vec::from(
                        &size_hex_bytes[size_hex_bytes.iter().position(|b| *b != b'0').unwrap()..],
                    );
                    chunk.extend_from_slice(b"\r\n");
                    chunk.append(&mut message);
                    chunk.extend_from_slice(b"\r\n");

                    crate::DEBUG!("\n[sending chunk]\n{}", chunk.escape_ascii());

                    conn.write_all(&chunk).await?;
                    conn.flush().await?;
                }
                conn.write_all(b"0\r\n\r\n").await?;
                conn.flush().await?;

                Ok(Upgrade::None)
            }

            #[cfg(all(feature = "ws", feature = "__rt_native__"))]
            Content::WebSocket(ws) => {
                let mut buf =
                    Vec::<u8>::with_capacity(self.status.line().len() + self.headers.size);
                unsafe {
                    crate::push_unchecked!(buf <- self.status.line());
                    self.headers.write_unchecked_to(&mut buf);
                }
                conn.write_all(&buf).await?;
                conn.flush().await?;

                Ok(Upgrade::WebSocket(ws))
            }
        }
    }
}

const _: () = {
    impl std::fmt::Debug for Response {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Response")
                .field("status", &self.status)
                .field("headers", &self.headers)
                .field("content", &self.content)
                .finish()
        }
    }

    impl PartialEq for Response {
        fn eq(&self, other: &Self) -> bool {
            self.status == other.status
                && self.headers == other.headers
                && self.content == other.content
        }
    }
};

#[cfg(feature = "nightly")]
#[cfg_attr(docsrs, doc(cfg(feature = "nightly")))]
const _: () = {
    use std::{convert::Infallible, ops::FromResidual};

    impl FromResidual<Result<Infallible, Response>> for Response {
        fn from_residual(residual: Result<Infallible, Response>) -> Self {
            unsafe { residual.unwrap_err_unchecked() }
        }
    }

    #[cfg(test)]
    fn _try_response() {
        // compiles
        fn payload_serde_json_value(req: &crate::Request) -> Result<::serde_json::Value, Response> {
            let payload = req.payload.as_deref().ok_or_else(Response::BadRequest)?;
            let value = serde_json::from_slice::<serde_json::Value>(payload)
                .map_err(|_| Response::BadRequest())?;
            Ok(value)
        }
    }
};

#[cfg(feature = "rt_worker")]
const _: () = {
    impl From<Response> for ::worker::Response {
        #[inline(always)]
        fn from(this: Response) -> ::worker::Response {
            this.content
                .into_worker_response()
                .with_status(this.status.code())
                .with_headers(this.headers.into())
        }
    }

    impl From<worker::Error> for Response {
        fn from(err: worker::Error) -> Response {
            IntoResponse::into_response(err)
        }
    }
};

#[cfg(feature = "rt_lambda")]
const _: () = {
    use crate::x_lambda::LambdaResponse;
    use ::lambda_runtime::FunctionResponse;
    use ohkami_lib::Stream;
    use std::{convert::Infallible, pin::Pin};

    impl From<Response>
        for FunctionResponse<
            LambdaResponse,
            Pin<Box<dyn Stream<Item = Result<String, Infallible>> + Send>>,
        >
    {
        fn from(
            this: Response,
        ) -> FunctionResponse<
            LambdaResponse,
            Pin<Box<dyn Stream<Item = Result<String, Infallible>> + Send>>,
        > {
            let mut headers = this.headers;

            let cookies = headers
                .setcookie
                .take(/* remove `Set-Cookie`s from app's own headers */)
                .map(|box_vec_cow_str| {
                    let mut vec_string = Vec::with_capacity(box_vec_cow_str.len());
                    for cow_str in *box_vec_cow_str {
                        vec_string.push(cow_str.into_owned());
                    }
                    vec_string
                });

            match this.content {
                Content::None => FunctionResponse::BufferedResponse(LambdaResponse {
                    statusCode: this.status.code(),
                    headers,
                    cookies,
                    body: None,
                    isBase64Encoded: None,
                }),

                Content::Payload(p) => {
                    let (encoded, body) = if let Ok(s) = std::str::from_utf8(&p) {
                        (false, s.into())
                    } else {
                        (true, crate::util::base64_encode(&*p))
                    };

                    FunctionResponse::BufferedResponse(LambdaResponse {
                        statusCode: this.status.code(),
                        headers,
                        cookies,
                        body: Some(body),
                        isBase64Encoded: Some(encoded),
                    })
                }

                #[cfg(feature = "sse")]
                Content::Stream(stream) => {
                    FunctionResponse::StreamingResponse(::lambda_runtime::StreamResponse {
                        stream: Box::pin(stream.map(Result::<_, Infallible>::Ok)),
                        metadata_prelude: ::lambda_runtime::MetadataPrelude {
                            // `StatusCode` of `http` crate
                            status_code: unsafe {
                                TryFrom::<u16>::try_from(this.status.code()).unwrap_unchecked()
                            },
                            // `HeaderMap` of `http` crate
                            headers: FromIterator/*::<HeaderName, HeaderValue>*/::from_iter(
                                headers.into_iter().map(
                                    |(n, v): (&'static str, Cow<'static, str>)| {
                                        (
                                            TryFrom::<&str>::try_from(n).unwrap(),
                                            TryFrom::<String>::try_from(v.into_owned()).unwrap(),
                                        )
                                    },
                                ),
                            ),
                            #[allow(clippy::unwrap_or_default)]
                            cookies: cookies.unwrap_or_else(Vec::new),
                        },
                    })
                }
            }
        }
    }
};
