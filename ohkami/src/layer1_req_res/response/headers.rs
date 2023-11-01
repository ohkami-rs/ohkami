/* ref: https://developer.mozilla.org/ja/docs/Web/HTTP/Headers */
#![allow(non_snake_case)]
#![allow(unused)] // until ....

use std::{collections::BTreeMap, sync::OnceLock, borrow::Cow};
use crate::{layer0_lib::{now, IntoCows}};


struct Header(Option<Cow<'static, str>>);

pub trait HeaderValue {
    fn into_header_value(self) -> Option<Cow<'static, str>>;
} const _: () = {
    impl<S: IntoCows<'static>> HeaderValue for S {
        fn into_header_value(self) -> Option<Cow<'static, str>> {
            Some(self.into_cow())
        }
    }
    impl HeaderValue for Option<()> {
        fn into_header_value(self) -> Option<Cow<'static, str>> {
            None
        }
    }
};

macro_rules! ResponseHeaders {
    ($(
        $group:ident {
            $( $key:literal $scope:vis $name:ident( $arg:ident ), )*
        }
    )*) => {
        /// Headers in a response.
        /// 
        /// Expected values: &'static str, String, Cow<'static, str>, or `None`
        /// 
        /// - `None` clears value of the header
        /// - others set the header to thet value
        /// 
        /// <br/>
        /// 
        /// - Content-Type
        /// - Content-Length
        /// - Access-Control-*
        /// - headers related to WebSocket handshake
        /// 
        /// are managed by ohkami and MUST NOT be set by `.custom` ( `.custom` has to be used **ONLY** to set custom HTTP headers like `X-MyApp-Data: amazing` )
        pub struct ResponseHeaders {
            $( $group: bool, )*
            $($( $name: Header, )*)*
            custom: BTreeMap<&'static str, &'static str>,
            cors_str: &'static str,
        }

        impl ResponseHeaders {
            $($(
                $scope fn $name(&mut self, $arg: impl HeaderValue) -> &mut Self {
                    self.$group = true;
                    self.$name.0 = $arg.into_header_value();
                    self
                }
            )*)*

            pub(crate) fn new() -> Self {
                Self {
                    $( $group: false, )*
                    $($( $name: Header(None), )*)*
                    custom: BTreeMap::new(),
                    cors_str: ""
                }
            }

            pub(crate) fn cors(&mut self, cors_str: &'static str) -> &mut Self {
                self.cors_str = cors_str;
                self
            }

            pub(crate) fn to_string(&self) -> String {
                let __now__ = now();

                let mut h = format!("\
                    Date: {__now__}\r\n\
                    {}\
                ", self.cors_str);

                $(
                    if self.$group {
                        $(
                            if let Some(value) = self.$name.0 {
                                h.push_str($key);h.push_str(&value);h.push('\r');h.push('\n');
                            }
                        )*
                    }
                )*

                for (k, v) in &self.custom {
                    h.push_str(k);h.push(':');h.push(' ');h.push_str(v);h.push('\r');h.push('\n');
                }

                h
            }
        }
    };
} ResponseHeaders! {
    auth_cookie_security_context {
        // authentication
        "WWW-Authenticate: "                 pub WWWAuthenticate(challenge),
        // cookie
        "Set-Cookie: "                       pub SetCookie(cookie_and_directives),
        // security
        "X-Frame-Options: "                  pub XFrameOptions(DENY_or_SAMEORIGIN),
        // response context
        "Allow: "                            pub Allow(methods),
        "Server: "                           pub Server(product),
    }

    cache_proxy_others {
        // cache
        "Age: "                              pub Age(delta_seconds),
        "Cache-Control: "                    pub CacheControl(cache_control),
        "Expires: "                          pub Expires(http_date),
        // proxy
        "Via: "                              pub Via(via),
        // others
        "Alt-Srv: "                          pub AltSvc(alternative_services),
    }

    conditions {
        "Last-Modified: "                    pub(crate) LastModified(http_date),
        "ETag: "                             pub(crate) ETag(identical_string),
        "If-Match: "                         pub(crate) IfMatch(etag_values),
        "If-None-Match: "                    pub(crate) IfNoneMatch(etag_values),
        "If-Modified-Since: "                pub(crate) IfModifiedSince(http_date),
        "If-Unmodified-Since: "              pub(crate) IfUnmodifiedSince(http_date),
        "Vary: "                             pub(crate) Vary(header_names),
    }

    message_body_and_encoding {
        // message body
        "Content-Language: "                 pub ContentLanguage(language_tag),
        "Content-Location: "                 pub ContentLocation(url),
        "Content-Encoding: "                 pub(crate) ContentEncoding(algorithm),
        // transfer encoding
        "Tranfer-Encoding: "                 pub(crate) TransferEncoding(chunked_compress_deflate_gzip_identity),
        "Trailer: "                          pub(crate) Trailer(header_names),
    }
}

impl ResponseHeaders {
    /// set custom HTTP headers like `X-MyApp-Data: apple`
    pub fn custom(&mut self, key: &'static str, value: impl HeaderValue) -> &mut Self {
        match value.into_header_value() {
            Some(value) => {
                self.custom.entry(key)
                    .and_modify(|v| *v = &value)
                    .or_insert(&value);
                self
            }
            None => {
                self.custom.remove(key);
                self
            }
        }
    }
}

const _: () = {
    impl std::fmt::Debug for ResponseHeaders {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.to_string())
        }
    }
};
