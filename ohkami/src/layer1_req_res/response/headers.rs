/* ref: https://developer.mozilla.org/ja/docs/Web/HTTP/Headers */
#![allow(non_snake_case)]

use std::{collections::HashMap};


struct Header(Option<&'static str>);

pub trait HeaderValue {
    fn into_header_value(self) -> Option<&'static str>;
}
impl HeaderValue for &'static str {fn into_header_value(self) -> Option<&'static str> {Some(self)}}
impl HeaderValue for Option<&'static str> {fn into_header_value(self) -> Option<&'static str> {self}}

macro_rules! ResponseHeaders {
    ($(
        $group:ident {
            $( $key:literal $name:ident( $arg:ident ), )*
        }
    )*) => {
        pub struct ResponseHeaders {
            $( $group: bool, )*
            $($( $name: Header, )*)*
            custom: HashMap<&'static str, &'static str>,
        }

        impl ResponseHeaders {
            $($(
                pub fn $name(&mut self, $arg: impl HeaderValue) -> &mut Self {
                    self.$group = true;
                    self.$name.0 = $arg.into_header_value();
                    self
                }
            )*)*

            pub(crate) fn new() -> Self {
                Self {
                    $( $group: false, )*
                    $($( $name: Header(None), )*)*
                    custom: HashMap::new()
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

                h.push('\r'); h.push('\n'); h
            }
        }
    };
} ResponseHeaders! {
    auth_cookie_security_context {
        // authentication
        "WWW-Authenticate: "                 WWWAuthenticate(challenge),
        // cookie
        "Set-Cookie: "                       SetCookie(cookie_and_directives),
        // security
        "X-Frame-Options: "                  XFrameOptions(DENY_or_SAMEORIGIN),
        // response context
        "Allow: "                            Allow(methods),
        "Server: "                           Server(product),
    }

    cache_proxy_redirect_others {
        // cache
        "Age: "                              Age(delta_seconds),
        "Cache-Control: "                    CacheControl(cache_control),
        "Expires: "                          Expires(http_date),
        // proxy
        "Via: "                              Via(via),
        // redirect
        "Location: "                         Location(url),
        // others
        "Alt-Srv: "                          AltSvc(alternative_services),
    }

    conditions {
        "Last-Modified: "                    LastModified(http_date),
        "E-tag: "                            Etag(identical_string),
        "If-Match: "                         IfMatch(etag_values),
        "If-None-Match: "                    IfNoneMatch(etag_values),
        "If-Modified-Since: "                IfModifiedSince(http_date),
        "If-Unmodified-Since: "              IfUnmodifiedSince(http_date),
        "Vary: "                             Vary(header_names),
    }

    cors {
        "Access-Control-Allow-Origin: "      AccessControlAllowOrigin(origin),
        "Access-Control-Allow-Credentials: " AccessControlAllowCredentials(true_if_needed),
        "Access-Control-Allow-Headers: "     AccessControlAllowHeaders(headers),
        "Access-Control-Allow-Methods: "     AccessControlAllowMethods(methods),
        "Access-Control-Expose-Headers: "    AccessControlExposeHeaders(headers),
        "Access-Control-Max-Age: "           AccessControlMaxAge(delta_seconds),
    }

    message_body_and_encoding {
        // message body
        "Content-Encoding: "                 ContentEncoding(algorithm),
        "Content-Language: "                 ContentLanguage(language_tag),
        "Content-Location: "                 ContentLocation(url),
        // transfer encoding
        "Tranfer-Encoding: "                 TransferEncoding(chunked_compress_deflate_gzip_identity),
        "Trailer: "                          Trailer(header_names),
    }
}

impl ResponseHeaders {
    pub fn costom(&mut self, key: &'static str, value: impl HeaderValue) {
        match value.into_header_value() {
            Some(value) => {self.custom.insert(key, value);}
            None        => {self.custom.remove(key);}
        }
    }
}
