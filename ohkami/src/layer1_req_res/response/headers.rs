/* ref: https://developer.mozilla.org/ja/docs/Web/HTTP/Headers */
#![allow(non_snake_case)]
#![allow(unused)] // until ....

use std::{collections::BTreeMap, sync::OnceLock};
use crate::{layer0_lib::now, CORSstr};


struct Header(Option<&'static str>);

pub trait HeaderValue {
    fn into_header_value(self) -> Option<&'static str>;
}
impl HeaderValue for &'static str {fn into_header_value(self) -> Option<&'static str> {Some(self)}}
impl HeaderValue for Option<&'static str> {fn into_header_value(self) -> Option<&'static str> {self}}


macro_rules! ResponseHeaders {
    ($(
        $group:ident {
            $( $key:literal $scope:vis $name:ident( $arg:ident ), )*
        }
    )*) => {
        /// Headers in a response.
        pub struct ResponseHeaders {
            $( $group: bool, )*
            $($( $name: Header, )*)*
            custom: BTreeMap<&'static str, &'static str>,
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
                    custom: BTreeMap::new()
                }
            }

            pub(crate) fn to_string(&self) -> String {
                let mut h = format!("Date: {}\r\n", now());

                if let Some(cors) = crate::CORS() {
                    h.push_str(CORSstr());h.push('\r');h.push('\n');
                }

                $(
                    if self.$group {
                        $(
                            if let Some(value) = self.$name.0 {
                                h.push_str($key);
                                h.push_str(value);
                                h.push('\r'); h.push('\n');
                            }
                        )*
                    }
                )*

                for (k, v) in &self.custom {
                    h.push_str(k);
                    h.push(':'); h.push(' ');
                    h.push_str(v);
                    h.push('\r'); h.push('\n');
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
        "Content-Encoding: "                 pub ContentEncoding(algorithm),
        "Content-Language: "                 pub ContentLanguage(language_tag),
        "Content-Location: "                 pub ContentLocation(url),
        // transfer encoding
        "Tranfer-Encoding: "                 pub(crate) TransferEncoding(chunked_compress_deflate_gzip_identity),
        "Trailer: "                          pub(crate) Trailer(header_names),
    }
}

impl ResponseHeaders {
    pub fn custom(&mut self, key: &'static str, value: impl HeaderValue) -> &mut Self {
        match value.into_header_value() {
            Some(value) => {
                self.custom.entry(key)
                    .and_modify(|v| *v = value)
                    .or_insert(value);
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
