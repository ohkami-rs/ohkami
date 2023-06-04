/* ref: https://developer.mozilla.org/ja/docs/Web/HTTP/Headers */
#![allow(non_snake_case)]

struct Header(Option<&'static str>);

pub trait HeaderValue {
    fn into_header_value(self) -> Option<&'static str>;
}
impl HeaderValue for &'static str {fn into_header_value(self) -> Option<&'static str> {Some(self)}}
impl HeaderValue for Option<&'static str> {fn into_header_value(self) -> Option<&'static str> {self}}

macro_rules! ResponseHeaders {
    ($(
        $key_literal:literal $visibility:vis $name:ident( $arg:ident )
    ),*) => {
        pub struct ResponseHeaders {
            $(
                $name: Header,
            )*
        }
        impl ResponseHeaders {
            $(
                $visibility fn $name<Value: HeaderValue>(&mut self, $arg: Value) -> &mut Self {
                    self.$name.0 = $arg.into_header_value();
                    self
                }
            )*
            pub(crate) fn to_string(&self) -> String {
                let mut h = String::with_capacity(256);
                $(
                    if let Some(value) = self.$name.0 {
                        h.push_str($key_literal);
                        h.push_str(value);
                        h.push('\r');h.push('\n')
                    }
                )*
                h.push('\r'); h.push('\n'); h
            }
        }
    };
} ResponseHeaders! {
    // authentication
    "WWW-Authenticate: "                 pub WWWAuthenticate(challenge),
    "Authorization: "                    pub Authorization(type_and_credentials),

    // cache
    "Age: "                              pub Age(seconds),
    "Cache-Control: "                    pub CacheControl(cache_control),
    "Expires: "                          pub Expires(http_date),

    // conditions
    "Last-Modified: "                    pub LastModified(http_date),
    "E-Tag: "                            pub Etag(identical_string),
    "If-Match: "                         pub IfMatch(etag_values),
    "If-None-Match: "                    pub IfNoneMatch(etag_values),
    "If-Modified-Since: "                pub IfModifiedSince(http_date),
    "If-Unmodified-Since: "              pub IfUnmodifiedSince(http_date),
    "Vary: "                             pub Vary(header_names),

    // connection managing
    "Connection: "                       pub Connection(close_or_headers),
    "Keep-Alive: "                       pub KeepAlive(timeout_and_max),

    // cookie
    "Set-Cookie: "                       pub SetCookie(cookie_and_directives),

    // cors
    "Access-Control-Allow-Origin: "      pub AccessControlAllowOrigin(origin),
    "Access-Control-Allow-Credentials: " pub AccessControlAllowCredentials(true_if_needed),
    "Access-Control-Allow-Headers: "     pub AccessControlAllowHeaders(headers),
    "Access-Control-Allow-Methods: "     pub AccessControlAllowMethods(methods),
    "Access-Control-Expose-Headers: "    pub AccessControlExposeHeaders(headers),
    "Access-Control-Max-Age: "           pub AccessControlMaxAge(delta_seconds),

    // message body
    "Content-Encoding: "                 pub ContentEncoding(algoeithm),
    "Content-Language: "                 pub ContentLanguage(language_tag),
    "Content-Location: "                 pub ContentLocation(url),

    // proxy
    "Via: "                              pub Via(via),

    // redirect
    "Location: "                         pub Location(url),

    // response context
    "Allow: "                            pub Allow(methods),
    "Server: "                           pub Server(product),

    // security
    "X-Frame-Options: "                  pub XFrameOptions(DENY_or_SAMEORIGIN),

    // reansfer encoding
    "Transfer-Encoding: "                pub TransferEncoding(chunked_compress_deflate_gzip_identity),
    "Trailer: "                          pub Trailer(header_names),

    // others
    "Alt-Svc: "                          pub AltSvc(alternative_services),
    "Date: "                             pub(crate) Date(now)
}
