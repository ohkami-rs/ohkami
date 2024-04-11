#![feature(test)]
extern crate test;

// use test::black_box;
fn black_box<T>(t: T) -> T {t}

use ohkami::__internal__::ResponseHeaders;
use http::{header, HeaderMap, HeaderName, HeaderValue};
use ohkami_benches::
    header_map::HeaderMap as MyHeaderMap;
use ohkami_benches::response_headers::{
    fxmap::FxMap,
    heap_ohkami_headers::HeapOhkamiHeaders,
    heap_ohkami_headers_nosize::HeapOhkamiHeadersWithoutSize,
};




#[bench] fn insert_ohkami(b: &mut test::Bencher) {
    let mut h = ResponseHeaders::_new();
    b.iter(|| {
        h.set()
            .AccessControlAllowCredentials(black_box("true"))
            .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
            .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
            .AccessControlAllowMethods(black_box("POST,GET,OPTIONS,DELETE"))
            .AccessControlMaxAge(black_box("86400"))
            .Vary(black_box("Origin"))
            .Server(black_box("ohkami"))
            .Connection(black_box("Keep-Alive"))
            .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
            .Via(black_box("HTTP/1.1 GWA"))
            .AltSvc(black_box("h2=\":433\"; ma=2592000;"))
            .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
            .ReferrerPolicy(black_box("same-origin"))
            .XFrameOptions(black_box("DENY"))
            .custom("x-myapp-data", black_box("myappdata; excellent"))
            .custom("something", black_box("anything"))
        ;
    });
}

#[bench] fn insert_heap_ohkami(b: &mut test::Bencher) {
    let mut h = HeapOhkamiHeaders::new();
    b.iter(|| {
        h.set()
            .AccessControlAllowCredentials(black_box("true"))
            .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
            .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
            .AccessControlAllowMethods(black_box("POST,GET,OPTIONS,DELETE"))
            .AccessControlMaxAge(black_box("86400"))
            .Vary(black_box("Origin"))
            .Server(black_box("ohkami"))
            .Connection(black_box("Keep-Alive"))
            .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
            .Via(black_box("HTTP/1.1 GWA"))
            .AltSvc(black_box("h2=\":433\"; ma=2592000;"))
            .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
            .ReferrerPolicy(black_box("same-origin"))
            .XFrameOptions(black_box("DENY"))
            .custom("x-myapp-data", black_box("myappdata; excellent"))
            .custom("something", black_box("anything"))
        ;
    });
}
#[bench] fn insert_heap_ohkami_nosize(b: &mut test::Bencher) {
    let mut h = HeapOhkamiHeadersWithoutSize::new();
    b.iter(|| {
        h.set()
            .AccessControlAllowCredentials(black_box("true"))
            .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
            .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
            .AccessControlAllowMethods(black_box("POST,GET,OPTIONS,DELETE"))
            .AccessControlMaxAge(black_box("86400"))
            .Vary(black_box("Origin"))
            .Server(black_box("ohkami"))
            .Connection(black_box("Keep-Alive"))
            .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
            .Via(black_box("HTTP/1.1 GWA"))
            .AltSvc(black_box("h2=\":433\"; ma=2592000;"))
            .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
            .ReferrerPolicy(black_box("same-origin"))
            .XFrameOptions(black_box("DENY"))
            .custom("x-myapp-data", black_box("myappdata; excellent"))
            .custom("something", black_box("anything"))
        ;
    });
}

#[bench] fn insert_http(b: &mut test::Bencher) {
    let mut h = HeaderMap::new();
    b.iter(|| {
        h.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static(black_box("true")));
        h.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static(black_box("X-Custom-Header,Upgrade-Insecure-Requests")));
        h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static(black_box("https://foo.bar.org")));
        h.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static(black_box("POST,GET,OPTIONS,DELETE")));
        h.insert(header::ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static(black_box("86400")));
        h.insert(header::VARY, HeaderValue::from_static(black_box("Origin")));
        h.insert(header::SERVER, HeaderValue::from_static(black_box("ohkami")));
        h.insert(header::CONNECTION, HeaderValue::from_static(black_box("Keep-Alive")));
        h.insert(header::DATE, HeaderValue::from_static(black_box("Wed, 21 Oct 2015 07:28:00 GMT")));
        h.insert(header::VIA, HeaderValue::from_static(black_box("HTTP/1.1 GWA")));
        h.insert(header::ALT_SVC, HeaderValue::from_static(black_box("h2=\":433\"; ma=2592000;")));
        h.insert(header::PROXY_AUTHENTICATE, HeaderValue::from_static(black_box("Basic realm=\"Access to the internal site\"")));
        h.insert(header::REFERRER_POLICY, HeaderValue::from_static("same-origin"));
        h.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
        h.insert(HeaderName::from_static("x-myapp-data"), HeaderValue::from_static(black_box("myappdata; excellent")));
        h.insert(HeaderName::from_static("something"), HeaderValue::from_static(black_box("anything")));
    });
}

#[bench] fn insert_fxmap(b: &mut test::Bencher) {
    let mut h = FxMap::new();
    b.iter(|| {
        h
            .insert("Access-Control-Allow-Credentials", black_box("true"))
            .insert("Access-Control-Allow-Headers", black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
            .insert("Access-Control-Allow-Origin", black_box("https://foo.bar.org"))
            .insert("Access-Control-Max-Age", black_box("86400"))
            .insert("Vary", black_box("Origin"))
            .insert("Server", black_box("ohkami"))
            .insert("Connection", black_box("Keep-Alive"))
            .insert("Date", black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
            .insert("Alt-Svc", black_box("h2=\":433\"; ma=2592000"))
            .insert("Proxy-Authenticate", black_box("Basic realm=\"Access to the internal site\""))
            .insert("Referer-Policy", black_box("same-origin"))
            .insert("X-Frame-Options", black_box("DEBY"))
            .insert("x-myapp-data", black_box("myappdata; excellent"))
            .insert("something", black_box("anything"))
        ;
    });
}

#[bench] fn insert_headermap(b: &mut test::Bencher) {
    let mut h = MyHeaderMap::new();
    b.iter(|| {
        h.set()
            .AccessControlAllowCredentials(black_box("true"))
            .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
            .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
            .AccessControlMaxAge(black_box("86400"))
            .Vary(black_box("Origin"))
            .Server(black_box("ohkami"))
            .Connection(black_box("Keep-Alive"))
            .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
            .AltSvc(black_box("h2=\":433\"; ma=2592000"))
            .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
            .ReferrerPolicy(black_box("same-origin"))
            .XFrameOptions(black_box("DEBY"))
            .custom("x-myapp-data", black_box("myappdata; excellent"))
            .custom("something", black_box("anything"))
        ;
    });
}




#[bench] fn remove_ohkami(b: &mut test::Bencher) {
    let mut h = ResponseHeaders::_new();
    h.set()
        .AccessControlAllowCredentials(black_box("true"))
        .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
        .AccessControlAllowMethods(black_box("POST,GET,OPTIONS,DELETE"))
        .AccessControlMaxAge(black_box("86400"))
        .Vary(black_box("Origin"))
        .Server(black_box("ohkami"))
        .Connection(black_box("Keep-Alive"))
        .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .Via(black_box("HTTP/1.1 GWA"))
        .AltSvc(black_box("h2=\":433\"; ma=2592000;"))
        .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
        .ReferrerPolicy(black_box("same-origin"))
        .XFrameOptions(black_box("DENY"))
        .custom("x-myapp-data", black_box("myappdata; excellent"))
        .custom("something", black_box("anything"))
    ;

    b.iter(|| {
        h.set()
            .AccessControlAllowCredentials(black_box(None))
            .AccessControlAllowHeaders(black_box(None))
            .AccessControlAllowOrigin(black_box(None))
            .AccessControlAllowMethods(black_box(None))
            .AccessControlMaxAge(black_box(None))
            .Vary(black_box(None))
            .Server(black_box(None))
            .Connection(black_box(None))
            .Date(black_box(None))
            .Via(black_box(None))
            .AltSvc(black_box(None))
            .ProxyAuthenticate(black_box(None))
            .ReferrerPolicy(black_box(None))
            .XFrameOptions(black_box(None))
            .custom("x-myapp-data", black_box(None))
            .custom("something", black_box(None))
        ;
    });
}

#[bench] fn remove_heap_ohkami(b: &mut test::Bencher) {
    let mut h = HeapOhkamiHeaders::new();
    h.set()
        .AccessControlAllowCredentials(black_box("true"))
        .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
        .AccessControlAllowMethods(black_box("POST,GET,OPTIONS,DELETE"))
        .AccessControlMaxAge(black_box("86400"))
        .Vary(black_box("Origin"))
        .Server(black_box("ohkami"))
        .Connection(black_box("Keep-Alive"))
        .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .Via(black_box("HTTP/1.1 GWA"))
        .AltSvc(black_box("h2=\":433\"; ma=2592000;"))
        .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
        .ReferrerPolicy(black_box("same-origin"))
        .XFrameOptions(black_box("DENY"))
        .custom("x-myapp-data", black_box("myappdata; excellent"))
        .custom("something", black_box("anything"))
    ;

    b.iter(|| {
        h.set()
            .AccessControlAllowCredentials(black_box(None))
            .AccessControlAllowHeaders(black_box(None))
            .AccessControlAllowOrigin(black_box(None))
            .AccessControlAllowMethods(black_box(None))
            .AccessControlMaxAge(black_box(None))
            .Vary(black_box(None))
            .Server(black_box(None))
            .Connection(black_box(None))
            .Date(black_box(None))
            .Via(black_box(None))
            .AltSvc(black_box(None))
            .ProxyAuthenticate(black_box(None))
            .ReferrerPolicy(black_box(None))
            .XFrameOptions(black_box(None))
            .custom("x-myapp-data", black_box(None))
            .custom("something", black_box(None))
        ;
    });
}
#[bench] fn remove_heap_ohkami_nosize(b: &mut test::Bencher) {
    let mut h = HeapOhkamiHeadersWithoutSize::new();
    h.set()
        .AccessControlAllowCredentials(black_box("true"))
        .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
        .AccessControlAllowMethods(black_box("POST,GET,OPTIONS,DELETE"))
        .AccessControlMaxAge(black_box("86400"))
        .Vary(black_box("Origin"))
        .Server(black_box("ohkami"))
        .Connection(black_box("Keep-Alive"))
        .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .Via(black_box("HTTP/1.1 GWA"))
        .AltSvc(black_box("h2=\":433\"; ma=2592000;"))
        .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
        .ReferrerPolicy(black_box("same-origin"))
        .XFrameOptions(black_box("DENY"))
        .custom("x-myapp-data", black_box("myappdata; excellent"))
        .custom("something", black_box("anything"))
    ;

    b.iter(|| {
        h.set()
            .AccessControlAllowCredentials(black_box(None))
            .AccessControlAllowHeaders(black_box(None))
            .AccessControlAllowOrigin(black_box(None))
            .AccessControlAllowMethods(black_box(None))
            .AccessControlMaxAge(black_box(None))
            .Vary(black_box(None))
            .Server(black_box(None))
            .Connection(black_box(None))
            .Date(black_box(None))
            .Via(black_box(None))
            .AltSvc(black_box(None))
            .ProxyAuthenticate(black_box(None))
            .ReferrerPolicy(black_box(None))
            .XFrameOptions(black_box(None))
            .custom("x-myapp-data", black_box(None))
            .custom("something", black_box(None))
        ;
    });
}

#[bench] fn remove_http(b: &mut test::Bencher) {
    let mut h = HeaderMap::new();
    h.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static(black_box("true")));
    h.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static(black_box("X-Custom-Header,Upgrade-Insecure-Requests")));
    h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static(black_box("https://foo.bar.org")));
    h.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static(black_box("POST,GET,OPTIONS,DELETE")));
    h.insert(header::ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static(black_box("86400")));
    h.insert(header::VARY, HeaderValue::from_static(black_box("Origin")));
    h.insert(header::SERVER, HeaderValue::from_static(black_box("ohkami")));
    h.insert(header::CONNECTION, HeaderValue::from_static(black_box("Keep-Alive")));
    h.insert(header::DATE, HeaderValue::from_static(black_box("Wed, 21 Oct 2015 07:28:00 GMT")));
    h.insert(header::VIA, HeaderValue::from_static(black_box("HTTP/1.1 GWA")));
    h.insert(header::ALT_SVC, HeaderValue::from_static(black_box("h2=\":433\"; ma=2592000;")));
    h.insert(header::PROXY_AUTHENTICATE, HeaderValue::from_static(black_box("Basic realm=\"Access to the internal site\"")));
    h.insert(header::REFERRER_POLICY, HeaderValue::from_static("same-origin"));
    h.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    h.insert(HeaderName::from_static("x-myapp-data"), HeaderValue::from_static(black_box("myappdata; excellent")));
    h.insert(HeaderName::from_static("something"), HeaderValue::from_static(black_box("anything")));

    b.iter(|| {
        h.remove(header::ACCESS_CONTROL_ALLOW_CREDENTIALS);
        h.remove(header::ACCESS_CONTROL_ALLOW_HEADERS);
        h.remove(header::ACCESS_CONTROL_ALLOW_ORIGIN);
        h.remove(header::ACCESS_CONTROL_ALLOW_METHODS);
        h.remove(header::ACCESS_CONTROL_MAX_AGE);
        h.remove(header::VARY);
        h.remove(header::SERVER);
        h.remove(header::CONNECTION);
        h.remove(header::DATE);
        h.remove(header::VIA);
        h.remove(header::ALT_SVC);
        h.remove(header::PROXY_AUTHENTICATE);
        h.remove(header::REFERRER_POLICY);
        h.remove(header::X_FRAME_OPTIONS);
        h.remove(HeaderName::from_static("x-myapp-data"));
        h.remove(HeaderName::from_static("something"));
    });
}

#[bench] fn remove_fxmap(b: &mut test::Bencher) {
    let mut h = FxMap::new();
    h
        .insert("Access-Control-Allow-Credentials", black_box("true"))
        .insert("Access-Control-Allow-Headers", black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .insert("Access-Control-Allow-Origin", black_box("https://foo.bar.org"))
        .insert("Access-Control-Max-Age", black_box("86400"))
        .insert("Vary", black_box("Origin"))
        .insert("Server", black_box("ohkami"))
        .insert("Connection", black_box("Keep-Alive"))
        .insert("Date", black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .insert("Alt-Svc", black_box("h2=\":433\"; ma=2592000"))
        .insert("Proxy-Authenticate", black_box("Basic realm=\"Access to the internal site\""))
        .insert("Referer-Policy", black_box("same-origin"))
        .insert("X-Frame-Options", black_box("DEBY"))
        .insert("x-myapp-data", black_box("myappdata; excellent"))
        .insert("something", black_box("anything"))
    ;

    b.iter(|| {
        h
            .remove("Access-Control-Allow-Credentials")
            .remove("Access-Control-Allow-Headers")
            .remove("Access-Control-Allow-Origin")
            .remove("Access-Control-Max-Age")
            .remove("Vary")
            .remove("Server")
            .remove("Connection")
            .remove("Date")
            .remove("Alt-Svc")
            .remove("Proxy-Authenticate")
            .remove("Referer-Policy")
            .remove("X-Frame-Options")
            .remove("x-myapp-data")
            .remove("something")
        ;
    });
}

#[bench] fn remove_headermap(b: &mut test::Bencher) {
    let mut h = MyHeaderMap::new();
    
    h.set()
        .AccessControlAllowCredentials(black_box("true"))
        .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
        .AccessControlMaxAge(black_box("86400"))
        .Vary(black_box("Origin"))
        .Server(black_box("ohkami"))
        .Connection(black_box("Keep-Alive"))
        .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .AltSvc(black_box("h2=\":433\"; ma=2592000"))
        .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
        .ReferrerPolicy(black_box("same-origin"))
        .XFrameOptions(black_box("DEBY"))
        .custom("x-myapp-data", black_box("myappdata; excellent"))
        .custom("something", black_box("anything"))
    ;

    b.iter(|| {
        h.set()
            .AccessControlAllowCredentials(black_box(None))
            .AccessControlAllowHeaders(black_box(None))
            .AccessControlAllowOrigin(black_box(None))
            .AccessControlAllowMethods(black_box(None))
            .AccessControlMaxAge(black_box(None))
            .Vary(black_box(None))
            .Server(black_box(None))
            .Connection(black_box(None))
            .Date(black_box(None))
            .Via(black_box(None))
            .AltSvc(black_box(None))
            .ProxyAuthenticate(black_box(None))
            .ReferrerPolicy(black_box(None))
            .XFrameOptions(black_box(None))
            .custom("x-myapp-data", black_box(None))
            .custom("something", black_box(None))
        ;
    });
}




#[bench] fn write_ohkami(b: &mut test::Bencher) {
    let mut h = ResponseHeaders::_new();
    h.set()
        .AccessControlAllowCredentials(black_box("true"))
        .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
        .AccessControlAllowMethods(black_box("POST,GET,OPTIONS,DELETE"))
        .AccessControlMaxAge(black_box("86400"))
        .Vary(black_box("Origin"))
        .Server(black_box("ohkami"))
        .Connection(black_box("Keep-Alive"))
        .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .Via(black_box("HTTP/1.1 GWA"))
        .AltSvc(black_box("h2=\":433\"; ma=2592000;"))
        .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
        .ReferrerPolicy(black_box("same-origin"))
        .XFrameOptions(black_box("DENY"))
        .custom("x-myapp-data", black_box("myappdata; excellent"))
        .custom("something", black_box("anything"))
    ;

    let mut buf = Vec::new();
    b.iter(|| {
        h.write_ref_to(&mut buf);
    });
}

#[bench] fn write_heap_ohkami(b: &mut test::Bencher) {
    let mut h = HeapOhkamiHeaders::new();
    h.set()
        .AccessControlAllowCredentials(black_box("true"))
        .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
        .AccessControlAllowMethods(black_box("POST,GET,OPTIONS,DELETE"))
        .AccessControlMaxAge(black_box("86400"))
        .Vary(black_box("Origin"))
        .Server(black_box("ohkami"))
        .Connection(black_box("Keep-Alive"))
        .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .Via(black_box("HTTP/1.1 GWA"))
        .AltSvc(black_box("h2=\":433\"; ma=2592000;"))
        .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
        .ReferrerPolicy(black_box("same-origin"))
        .XFrameOptions(black_box("DENY"))
        .custom("x-myapp-data", black_box("myappdata; excellent"))
        .custom("something", black_box("anything"))
    ;

    let mut buf = Vec::new();
    b.iter(|| {
        h.write_to(&mut buf);
    });
}
#[bench] fn write_heap_ohkami_only_standards(b: &mut test::Bencher) {
    let mut h = HeapOhkamiHeaders::new();
    h.set()
        .AccessControlAllowCredentials(black_box("true"))
        .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
        .AccessControlAllowMethods(black_box("POST,GET,OPTIONS,DELETE"))
        .AccessControlMaxAge(black_box("86400"))
        .Vary(black_box("Origin"))
        .Server(black_box("ohkami"))
        .Connection(black_box("Keep-Alive"))
        .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .Via(black_box("HTTP/1.1 GWA"))
        .AltSvc(black_box("h2=\":433\"; ma=2592000;"))
        .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
        .ReferrerPolicy(black_box("same-origin"))
        .XFrameOptions(black_box("DENY"))
        .custom("x-myapp-data", black_box("myappdata; excellent"))
        .custom("something", black_box("anything"))
    ;

    let mut buf = Vec::new();
    b.iter(|| {
        h.write_standards_to(&mut buf);
    });
}#[bench] fn write_heap_ohkami_nosize(b: &mut test::Bencher) {
    let mut h = HeapOhkamiHeadersWithoutSize::new();
    h.set()
        .AccessControlAllowCredentials(black_box("true"))
        .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
        .AccessControlAllowMethods(black_box("POST,GET,OPTIONS,DELETE"))
        .AccessControlMaxAge(black_box("86400"))
        .Vary(black_box("Origin"))
        .Server(black_box("ohkami"))
        .Connection(black_box("Keep-Alive"))
        .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .Via(black_box("HTTP/1.1 GWA"))
        .AltSvc(black_box("h2=\":433\"; ma=2592000;"))
        .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
        .ReferrerPolicy(black_box("same-origin"))
        .XFrameOptions(black_box("DENY"))
        .custom("x-myapp-data", black_box("myappdata; excellent"))
        .custom("something", black_box("anything"))
    ;

    let mut buf = Vec::new();
    b.iter(|| {
        h.write_to(&mut buf);
    });
}
#[bench] fn write_heap_ohkami_only_standards_nosize(b: &mut test::Bencher) {
    let mut h = HeapOhkamiHeadersWithoutSize::new();
    h.set()
        .AccessControlAllowCredentials(black_box("true"))
        .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
        .AccessControlAllowMethods(black_box("POST,GET,OPTIONS,DELETE"))
        .AccessControlMaxAge(black_box("86400"))
        .Vary(black_box("Origin"))
        .Server(black_box("ohkami"))
        .Connection(black_box("Keep-Alive"))
        .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .Via(black_box("HTTP/1.1 GWA"))
        .AltSvc(black_box("h2=\":433\"; ma=2592000;"))
        .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
        .ReferrerPolicy(black_box("same-origin"))
        .XFrameOptions(black_box("DENY"))
        .custom("x-myapp-data", black_box("myappdata; excellent"))
        .custom("something", black_box("anything"))
    ;

    let mut buf = Vec::new();
    b.iter(|| {
        h.write_standards_to(&mut buf);
    });
}

#[bench] fn write_http(b: &mut test::Bencher) {
    let mut h = HeaderMap::new();
    h.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static(black_box("true")));
    h.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static(black_box("X-Custom-Header,Upgrade-Insecure-Requests")));
    h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static(black_box("https://foo.bar.org")));
    h.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static(black_box("POST,GET,OPTIONS,DELETE")));
    h.insert(header::ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static(black_box("86400")));
    h.insert(header::VARY, HeaderValue::from_static(black_box("Origin")));
    h.insert(header::SERVER, HeaderValue::from_static(black_box("ohkami")));
    h.insert(header::CONNECTION, HeaderValue::from_static(black_box("Keep-Alive")));
    h.insert(header::DATE, HeaderValue::from_static(black_box("Wed, 21 Oct 2015 07:28:00 GMT")));
    h.insert(header::VIA, HeaderValue::from_static(black_box("HTTP/1.1 GWA")));
    h.insert(header::ALT_SVC, HeaderValue::from_static(black_box("h2=\":433\"; ma=2592000;")));
    h.insert(header::PROXY_AUTHENTICATE, HeaderValue::from_static(black_box("Basic realm=\"Access to the internal site\"")));
    h.insert(header::REFERRER_POLICY, HeaderValue::from_static("same-origin"));
    h.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    h.insert(HeaderName::from_static("x-myapp-data"), HeaderValue::from_static(black_box("myappdata; excellent")));
    h.insert(HeaderName::from_static("something"), HeaderValue::from_static(black_box("anything")));

    let mut buf = Vec::new();
    b.iter(|| {
        for (k, v) in h.iter() {
            buf.extend_from_slice(k.as_str().as_bytes());
            buf.extend(b": ");
            buf.extend(v.as_bytes());
            buf.extend(b"\r\n");
        }
        buf.extend(b"\r\n");
    });
}

#[bench] fn write_fxmap(b: &mut test::Bencher) {
    let mut h = FxMap::new();
    h
        .insert("Access-Control-Allow-Credentials", black_box("true"))
        .insert("Access-Control-Allow-Headers", black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .insert("Access-Control-Allow-Origin", black_box("https://foo.bar.org"))
        .insert("Access-Control-Max-Age", black_box("86400"))
        .insert("Vary", black_box("Origin"))
        .insert("Server", black_box("ohkami"))
        .insert("Connection", black_box("Keep-Alive"))
        .insert("Date", black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .insert("Alt-Svc", black_box("h2=\":433\"; ma=2592000"))
        .insert("Proxy-Authenticate", black_box("Basic realm=\"Access to the internal site\""))
        .insert("Referer-Policy", black_box("same-origin"))
        .insert("X-Frame-Options", black_box("DEBY"))
        .insert("x-myapp-data", black_box("myappdata; excellent"))
        .insert("something", black_box("anything"))
    ;

    let mut buf = Vec::new();
    b.iter(|| {
        h.write_to(&mut buf);
    });
}

#[bench] fn write_headermap(b: &mut test::Bencher) {
    let mut h = MyHeaderMap::new();
    h.set()
        .AccessControlAllowCredentials(black_box("true"))
        .AccessControlAllowHeaders(black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .AccessControlAllowOrigin(black_box("https://foo.bar.org"))
        .AccessControlMaxAge(black_box("86400"))
        .Vary(black_box("Origin"))
        .Server(black_box("ohkami"))
        .Connection(black_box("Keep-Alive"))
        .Date(black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .AltSvc(black_box("h2=\":433\"; ma=2592000"))
        .ProxyAuthenticate(black_box("Basic realm=\"Access to the internal site\""))
        .ReferrerPolicy(black_box("same-origin"))
        .XFrameOptions(black_box("DEBY"))
        .custom("x-myapp-data", black_box("myappdata; excellent"))
        .custom("something", black_box("anything"))
    ;

    let mut buf = Vec::new();
    b.iter(|| {
        h.write_to(&mut buf);
    });
}
