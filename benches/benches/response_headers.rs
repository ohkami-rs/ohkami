#![feature(test)]
extern crate test;

// use test::black_box;
fn black_box<T>(t: T) -> T {t}

use ohkami::__internal__::ResponseHeaders;
use http::{header, HeaderMap, HeaderName, HeaderValue};
use ohkami_benches::header_hashbrown::{HeaderHashBrown, StandardHeader};
use ohkami_benches::header_map::HeaderMap as MyHeaderMap;
use ohkami_benches::response_headers::{
    fxmap::FxMap,
    // heap_ohkami_headers::HeapOhkamiHeaders,
    // heap_ohkami_headers_nosize::HeapOhkamiHeadersWithoutSize,
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
            .x("x-myapp-data", black_box("myappdata; excellent"))
            .x("something", black_box("anything"))
        ;
    });
}
/*
#[bench] fn insert_ohkami_only_standard(b: &mut test::Bencher) {
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
            .x("x-myapp-data", black_box("myappdata; excellent"))
            .x("something", black_box("anything"))
        ;
    });
}
#[bench] fn insert_heap_ohkami_only_standard(b: &mut test::Bencher) {
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
        ;
    });
}
*/

#[bench] fn insert_http_crate(b: &mut test::Bencher) {
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
            .insert("Access-Control-Allow-Methods", black_box("POST,GET,OPTIONS,DELETE"))
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
            .AccessControlAllowMethods(black_box("POST,GET,OPTIONS,DELETE"))
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

#[bench] fn insert_header_hashbrown(b: &mut test::Bencher) {
    let mut h = HeaderHashBrown::<true>::new();
    b.iter(|| {
        h
            .insert_standard_from_reqbytes(StandardHeader::AccessControlAllowCredentials, black_box(b"true"))
            .insert_standard_from_reqbytes(StandardHeader::AccessControlAllowHeaders, black_box(b"X-Custom-Header,Upgrade-Insecure-Requests"))
            .insert_standard_from_reqbytes(StandardHeader::AccessControlAllowOrigin, black_box(b"https://foo.bar.org"))
            .insert_standard_from_reqbytes(StandardHeader::AccessControlAllowMethods, black_box(b"POST,GET,OPTIONS,DELETE"))
            .insert_standard_from_reqbytes(StandardHeader::AccessControlMaxAge, black_box(b"86400"))
            .insert_standard_from_reqbytes(StandardHeader::Vary, black_box(b"Origin"))
            .insert_standard_from_reqbytes(StandardHeader::Server, black_box(b"ohkami"))
            .insert_standard_from_reqbytes(StandardHeader::Connection, black_box(b"Keep-Alive"))
            .insert_standard_from_reqbytes(StandardHeader::Date, black_box(b"Wed, 21 Oct 2015 07:28:00 GMT"))
            .insert_standard_from_reqbytes(StandardHeader::AltSvc, black_box(b"h2=\":433\"; ma=2592000"))
            .insert_standard_from_reqbytes(StandardHeader::ProxyAuthenticate, black_box(b"Basic realm=\"Access to the internal site\""))
            .insert_standard_from_reqbytes(StandardHeader::ReferrerPolicy, black_box(b"same-origin"))
            .insert_standard_from_reqbytes(StandardHeader::XFrameOptions, black_box(b"DEBY"))
            .insert_from_reqbytes(b"x-myapp-data", black_box(b"myappdata; excellent"))
            .insert_from_reqbytes(b"something", black_box(b"anything"))
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
        .x("x-myapp-data", black_box("myappdata; excellent"))
        .x("something", black_box("anything"))
    ;

    b.iter(|| {
        h.set()
            .AccessControlAllowCredentials(None)
            .AccessControlAllowHeaders(None)
            .AccessControlAllowOrigin(None)
            .AccessControlAllowMethods(None)
            .AccessControlMaxAge(None)
            .Vary(None)
            .Server(None)
            .Connection(None)
            .Date(None)
            .Via(None)
            .AltSvc(None)
            .ProxyAuthenticate(None)
            .ReferrerPolicy(None)
            .XFrameOptions(None)
            .x("x-myapp-data", None)
            .x("something", None)
        ;
    });
}
/*
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
*/

#[bench] fn remove_http_crate(b: &mut test::Bencher) {
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
        .insert("Access-Control-Allow-Methods", black_box("POST,GET,OPTIONS,DELETE"))
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
            .remove("Access-Control-Allow-Methods")
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
        .AccessControlAllowMethods(black_box("POST,GET,OPTIONS,DELETE"))
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
            .AccessControlAllowCredentials(None)
            .AccessControlAllowHeaders(None)
            .AccessControlAllowOrigin(None)
            .AccessControlAllowMethods(None)
            .AccessControlMaxAge(None)
            .Vary(None)
            .Server(None)
            .Connection(None)
            .Date(None)
            .Via(None)
            .AltSvc(None)
            .ProxyAuthenticate(None)
            .ReferrerPolicy(None)
            .XFrameOptions(None)
            .custom("x-myapp-data", None)
            .custom("something", None)
        ;
    });
}

#[bench] fn remove_header_hashbrown(b: &mut test::Bencher) {
    let mut h = HeaderHashBrown::<true>::new();
    h
        .insert_standard_from_reqbytes(StandardHeader::AccessControlAllowCredentials, black_box(b"true"))
        .insert_standard_from_reqbytes(StandardHeader::AccessControlAllowHeaders, black_box(b"X-Custom-Header,Upgrade-Insecure-Requests"))
        .insert_standard_from_reqbytes(StandardHeader::AccessControlAllowOrigin, black_box(b"https://foo.bar.org"))
        .insert_standard_from_reqbytes(StandardHeader::AccessControlAllowMethods, black_box(b"POST,GET,OPTIONS,DELETE"))
        .insert_standard_from_reqbytes(StandardHeader::AccessControlMaxAge, black_box(b"86400"))
        .insert_standard_from_reqbytes(StandardHeader::Vary, black_box(b"Origin"))
        .insert_standard_from_reqbytes(StandardHeader::Server, black_box(b"ohkami"))
        .insert_standard_from_reqbytes(StandardHeader::Connection, black_box(b"Keep-Alive"))
        .insert_standard_from_reqbytes(StandardHeader::Date, black_box(b"Wed, 21 Oct 2015 07:28:00 GMT"))
        .insert_standard_from_reqbytes(StandardHeader::AltSvc, black_box(b"h2=\":433\"; ma=2592000"))
        .insert_standard_from_reqbytes(StandardHeader::ProxyAuthenticate, black_box(b"Basic realm=\"Access to the internal site\""))
        .insert_standard_from_reqbytes(StandardHeader::ReferrerPolicy, black_box(b"same-origin"))
        .insert_standard_from_reqbytes(StandardHeader::XFrameOptions, black_box(b"DEBY"))
        .insert_from_reqbytes(b"x-myapp-data", black_box(b"myappdata; excellent"))
        .insert_from_reqbytes(b"something", black_box(b"anything"))
    ;

    b.iter(|| {
        h
            .remove_standard(StandardHeader::AccessControlAllowCredentials)
            .remove_standard(StandardHeader::AccessControlAllowHeaders)
            .remove_standard(StandardHeader::AccessControlAllowOrigin)
            .remove_standard(StandardHeader::AccessControlAllowMethods)
            .remove_standard(StandardHeader::AccessControlMaxAge)
            .remove_standard(StandardHeader::Vary)
            .remove_standard(StandardHeader::Server)
            .remove_standard(StandardHeader::Connection)
            .remove_standard(StandardHeader::Date)
            .remove_standard(StandardHeader::Via)
            .remove_standard(StandardHeader::AltSvc)
            .remove_standard(StandardHeader::ProxyAuthenticate)
            .remove_standard(StandardHeader::ReferrerPolicy)
            .remove_standard(StandardHeader::XFrameOptions)
            .remove("x-myapp-data")
            .remove("something")
        ;
    });
}



#[bench] fn write_03_ohkami(b: &mut test::Bencher) {
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
        .x("x-myapp-data", black_box("myappdata; excellent"))
        .x("something", black_box("anything"))
    ;

    let mut buf = Vec::new();
    b.iter(|| {
        h._write_to(&mut buf);
    });
}
/*
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
}
#[bench] fn write_heap_ohkami_nosize(b: &mut test::Bencher) {
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
*/

#[bench] fn write_02_http_crate(b: &mut test::Bencher) {
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

#[bench] fn write_03_fxmap(b: &mut test::Bencher) {
    let mut h = FxMap::new();
    h
        .insert("Access-Control-Allow-Credentials", black_box("true"))
        .insert("Access-Control-Allow-Headers", black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .insert("Access-Control-Allow-Origin", black_box("https://foo.bar.org"))
        .insert("Access-Conctrol-Allow-Methods", black_box("POST,GET,OPTIONS,DELETE"))
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

#[bench] fn write_04_headermap(b: &mut test::Bencher) {
    let mut h = MyHeaderMap::new();
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

#[bench] fn write_03_header_hashbrown(b: &mut test::Bencher) {
    let mut h = HeaderHashBrown::<true>::new();
    h
        .insert_standard_from_reqbytes(StandardHeader::AccessControlAllowCredentials, black_box(b"true"))
        .insert_standard_from_reqbytes(StandardHeader::AccessControlAllowHeaders, black_box(b"X-Custom-Header,Upgrade-Insecure-Requests"))
        .insert_standard_from_reqbytes(StandardHeader::AccessControlAllowOrigin, black_box(b"https://foo.bar.org"))
        .insert_standard_from_reqbytes(StandardHeader::AccessControlAllowMethods, black_box(b"POST,GET,OPTIONS,DELETE"))
        .insert_standard_from_reqbytes(StandardHeader::AccessControlMaxAge, black_box(b"86400"))
        .insert_standard_from_reqbytes(StandardHeader::Vary, black_box(b"Origin"))
        .insert_standard_from_reqbytes(StandardHeader::Server, black_box(b"ohkami"))
        .insert_standard_from_reqbytes(StandardHeader::Connection, black_box(b"Keep-Alive"))
        .insert_standard_from_reqbytes(StandardHeader::Date, black_box(b"Wed, 21 Oct 2015 07:28:00 GMT"))
        .insert_standard_from_reqbytes(StandardHeader::AltSvc, black_box(b"h2=\":433\"; ma=2592000"))
        .insert_standard_from_reqbytes(StandardHeader::ProxyAuthenticate, black_box(b"Basic realm=\"Access to the internal site\""))
        .insert_standard_from_reqbytes(StandardHeader::ReferrerPolicy, black_box(b"same-origin"))
        .insert_standard_from_reqbytes(StandardHeader::XFrameOptions, black_box(b"DEBY"))
        .insert_from_reqbytes(b"x-myapp-data", black_box(b"myappdata; excellent"))
        .insert_from_reqbytes(b"something", black_box(b"anything"))
    ;

    let mut buf = Vec::new();
    b.iter(|| {
        h.write_to(&mut buf);
    });
}
