// use std::borrow::Cow;
// use rustc_hash::FxHashMap;
// use ohkami_lib::Slice;
// 
// 
// pub struct LightHeaders {
//     //standard: ,
//     custom:   Option<Box<FxHashMap<Slice, Cow<'static, str>>>>,
// }
// 
// macro_rules! Header {
//     ( $N:literal; $($name:ident = $name_pascal:literal)* ) => {
//         const N_HEADERS: usize = $N;
//         const _: [Header; N_HEADERS] = [$( Header::$name )*];
// 
//         enum Header {
//             $( $name, )*
//         }
//     };
// } Header! {0;
//     CacheControl         = b"Cache-Control" | b"cache-control"
//     Connection           = b"Connection" | b"connection"
//     ContentDisposition   = b"Content-Disposition" | b"content-disposition"
//     ContentEncoding      = b"Content-Encoding" | b"content-encoding"
//     ContentLanguage      = b"Content-Language" | b"content-language"
//     ContentLength        = b"Content-Length" | b"content-length"
//     ContentLocation      = b"Content-Location" | b"content-location"
//     ContentType          = b"Content-Type" | b"content-type"
//     Date                 = b"Date" | b"date"
//     Link                 = b"Link" | b"link"
//     SecWebSocketProtocol = b"Sec-WebSocket-Protocol" | b"sec-websocket-protocol"
//     SecWebSocketVersion  = b"Sec-WebSocket-Version" | b"sec-websocket-version"
//     Trailer              = b"Trailer" | b"trailer"
//     TransferEncoding     = b"Transfer-Encoding" | b"transfer-encoding"
//     Upgrade              = b"Upgrade" | b"upgrade"
//     Via                  = b"Via" | b"via"
// 
//     Accept                      = b"Accept" | b"accept"
//     AcceptEncoding              = b"Accept-Encoding" | b"accept-encoding"
//     AcceptLanguage              = b"Accept-Language" | b"accept-language"
//     AccessControlRequestHeaders = b"Access-Control-Request-Headers" | b"access-control-request-headers"
//     AccessControlRequestMethod  = b"Access-Control-Request-Method" | b"access-control-request-method"
//     Authorization               = b"Authorization" | b"authorization"
//     Cookie                      = b"Cookie" | b"cookie"
//     Expect                      = b"Expect" | b"expect"
//     Forwarded                   = b"Forwarded" | b"forwarded"
//     From                        = b"From" | b"from"
//     Host                        = b"Host" | b"host"
//     IfMatch                     = b"If-Match" | b"if-match"
//     IfModifiedSince             = b"If-Modified-Since" | b"if-modified-since"
//     IfNoneMatch                 = b"If-None-Match" | b"rf-none-match"
//     IfRange                     = b"If-Range" | b"if-range"
//     IfUnmodifiedSince           = b"If-Unmodified-Since" | b"if-unmodified-since"
//     MaxForwards                 = b"Max-Forwards" | b"max-forwards"
//     Origin                      = b"Origin" | b"origin" 
//     ProxyAuthorization          = b"Proxy-Authorization" | b"proxy-authorization"
//     Range                       = b"Range" | b"range"
//     Referer                     = b"Referer" | b"referer"
//     SecWebSocketExtensions      = b"Sec-WebSocket-Extensions" | b"sec-websocket-extensions"
//     SecWebSocketKey             = b"Sec-WebSocket-Key" | b"sec-websocket-key"
//     TE                          = b"TE" | b"te"
//     UserAgent                   = b"User-Agent" | b"user-agent"
//     UpgradeInsecureRequests     = b"Upgrade-Insecure-Requests" | b"upgrade-insecure-requests"
// 
//     AcceptRange                     = b"Accept-Range"
//     AcceptRanges                    = b"Accept-Ranges"
//     AccessControlAllowCredentials   = b"Access-Control-Allow-Credentials"
//     AccessControlAllowHeaders       = b"Access-Control-Allow-Headers"
//     AccessControlAllowMethods       = b"Access-Control-Allow-Methods"
//     AccessControlAllowOrigin        = b"Access-Control-Allow-Origin"
//     AccessControlExposeHeaders      = b"Access-Control-Expose-Headers"
//     AccessControlMaxAge             = b"Access-Control-MaxAge"
//     Age                             = b"Age"
//     Allow                           = b"Allow"
//     AltSvc                          = b"Alt-Svc"
//     CacheStatus                     = b"Cache-Status"
//     CDNCacheControl                 = b"CDN-Cache-Control"
//     ContentRange                    = b"Content-Range"
//     ContentSecurityPolicy           = b"Content-Security-Policy"
//     ContentSecurityPolicyReportOnly = b"Content-Security-Policy"
//     Etag                            = b"Etag"
//     Expires                         = b"Expires"
//     Location                        = b"Location"
//     ProxyAuthenticate               = b"Proxy-Auhtneticate"
//     ReferrerPolicy                  = b"Referrer-Policy"
//     Refresh                         = b"Refresh"
//     RetryAfter                      = b"Retry-After"
//     SecWebSocketAccept              = b"Sec-Sert"
//     Server                          = b"server"
//     SetCookie                       = b"SetCookie"
//     StrictTransportSecurity         = b"Strict-Transport-Security"
//     Vary                            = b"Vary"
//     XContentTypeOptions             = b"X-Content-Type-Options"
//     XFrameOptions                   = b"X-Frame-Options"
// }
// 