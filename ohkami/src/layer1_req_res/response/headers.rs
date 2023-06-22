/* ref: https://developer.mozilla.org/ja/docs/Web/HTTP/Headers */
#![allow(non_snake_case)]

use std::{collections::BTreeMap};


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
                let mut h = format!("\
                    Connection: Keep-Alive\r\n\
                    Keep-Alive: timout=5\r\n\
                    Date: {}\r\n\
                ", crate::layer0_lib::now());

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
        "WWW-Authenticate: "                 pub(crate) WWWAuthenticate(challenge),
        // cookie
        "Set-Cookie: "                       pub(crate) SetCookie(cookie_and_directives),
        // security
        "X-Frame-Options: "                  pub(crate) XFrameOptions(DENY_or_SAMEORIGIN),
        // response context
        "Allow: "                            pub(crate) Allow(methods),
        "Server: "                           pub Server(product),
    }

    cache_proxy_redirect_others {
        // cache
        "Age: "                              pub(crate) Age(delta_seconds),
        "Cache-Control: "                    pub(crate) CacheControl(cache_control),
        "Expires: "                          pub(crate) Expires(http_date),
        // proxy
        "Via: "                              pub(crate) Via(via),
        // redirect
        "Location: "                         pub(crate) Location(url),
        // others
        "Alt-Srv: "                          pub(crate) AltSvc(alternative_services),
    }

    conditions {
        "Last-Modified: "                    pub(crate) LastModified(http_date),
        "E-tag: "                            pub(crate) Etag(identical_string),
        "If-Match: "                         pub(crate) IfMatch(etag_values),
        "If-None-Match: "                    pub(crate) IfNoneMatch(etag_values),
        "If-Modified-Since: "                pub(crate) IfModifiedSince(http_date),
        "If-Unmodified-Since: "              pub(crate) IfUnmodifiedSince(http_date),
        "Vary: "                             pub(crate) Vary(header_names),
    }

    cors {
        "Access-Control-Allow-Origin: "      pub(crate) AccessControlAllowOrigin(origin),
        "Access-Control-Allow-Credentials: " pub(crate) AccessControlAllowCredentials(true_if_needed),
        "Access-Control-Allow-Headers: "     pub(crate) AccessControlAllowHeaders(headers),
        "Access-Control-Allow-Methods: "     pub(crate) AccessControlAllowMethods(methods),
        "Access-Control-Expose-Headers: "    pub(crate) AccessControlExposeHeaders(headers),
        "Access-Control-Max-Age: "           pub(crate) AccessControlMaxAge(delta_seconds),
    }

    message_body_and_encoding {
        // message body
        "Content-Encoding: "                 pub(crate) ContentEncoding(algorithm),
        "Content-Language: "                 pub(crate) ContentLanguage(language_tag),
        "Content-Location: "                 pub(crate) ContentLocation(url),
        // transfer encoding
        "Tranfer-Encoding: "                 pub(crate) TransferEncoding(chunked_compress_deflate_gzip_identity),
        "Trailer: "                          pub(crate) Trailer(header_names),
    }
}

impl ResponseHeaders {
    pub fn costom(&mut self, key: &'static str, value: impl HeaderValue) {
        match value.into_header_value() {
            Some(value) => {
                self.custom.entry(key)
                    .and_modify(|v| *v = value)
                    .or_insert(value);
            }
            None => {
                self.custom.remove(key);
            }
        }
    }
}
