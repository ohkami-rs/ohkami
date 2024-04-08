#![feature(test)]
extern crate test;


use ohkami::__internal__::ResponseHeaders;
use http::{HeaderMap, header, HeaderValue};
use ohkami_benches::{fxmap::FxMap, header_map::ResponseHeaderMap};




#[bench] fn insert_ohkami(b: &mut test::Bencher) {
    let mut h = ResponseHeaders::init();
    b.iter(|| {
        h.set()
            .AccessControlAllowCredentials(test::black_box("true"))
            .AccessControlAllowHeaders(test::black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
            .AccessControlAllowOrigin(test::black_box("https://foo.bar.org"))
            .AccessControlAllowMethods(test::black_box("POST,GET,OPTIONS,DELETE"))
            .AccessControlMaxAge(test::black_box("86400"))
            .Vary(test::black_box("Origin"))
            .Server(test::black_box("ohkami"))
            .Connection(test::black_box("Keep-Alive"))
            .Date(test::black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
            .Via(test::black_box("HTTP/1.1 GWA"))
            .AltSvc(test::black_box("h2=\":433\"; ma=2592000;"))
            .ProxyAuthenticate(test::black_box("Basic realm=\"Access to the internal site\""))
            .ReferrerPolicy(test::black_box("same-origin"))
            .XFrameOptions(test::black_box("DENY"));
    });
}

#[bench] fn insert_http(b: &mut test::Bencher) {
    let mut h = HeaderMap::new();
    b.iter(|| {
        h.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static(test::black_box("true")));
        h.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static(test::black_box("X-Custom-Header,Upgrade-Insecure-Requests")));
        h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static(test::black_box("https://foo.bar.org")));
        h.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static(test::black_box("POST,GET,OPTIONS,DELETE")));
        h.insert(header::ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static(test::black_box("86400")));
        h.insert(header::VARY, HeaderValue::from_static(test::black_box("Origin")));
        h.insert(header::SERVER, HeaderValue::from_static(test::black_box("ohkami")));
        h.insert(header::CONNECTION, HeaderValue::from_static(test::black_box("Keep-Alive")));
        h.insert(header::DATE, HeaderValue::from_static(test::black_box("Wed, 21 Oct 2015 07:28:00 GMT")));
        h.insert(header::VIA, HeaderValue::from_static(test::black_box("HTTP/1.1 GWA")));
        h.insert(header::ALT_SVC, HeaderValue::from_static(test::black_box("h2=\":433\"; ma=2592000;")));
        h.insert(header::PROXY_AUTHENTICATE, HeaderValue::from_static(test::black_box("Basic realm=\"Access to the internal site\"")));
        h.insert(header::REFERRER_POLICY, HeaderValue::from_static("same-origin"));
        h.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    });
}

#[bench] fn insert_fxmap(b: &mut test::Bencher) {
    let mut h = FxMap::new();
    b.iter(|| {
        h
            .insert("Access-Control-Allow-Credentials", test::black_box("true"))
            .insert("Access-Control-Allow-Headers", test::black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
            .insert("Access-Control-Allow-Origin", test::black_box("https://foo.bar.org"))
            .insert("Access-Control-Max-Age", test::black_box("86400"))
            .insert("Vary", test::black_box("Origin"))
            .insert("Server", test::black_box("ohkami"))
            .insert("Connection", test::black_box("Keep-Alive"))
            .insert("Date", test::black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
            .insert("Alt-Svc", test::black_box("h2=\":433\"; ma=2592000"))
            .insert("Proxy-Authenticate", test::black_box("Basic realm=\"Access to the internal site\""))
            .insert("Referer-Policy", test::black_box("same-origin"))
            .insert("X-Frame-Options", test::black_box("DEBY"));
    });
}

#[bench] fn insert_headermap(b: &mut test::Bencher) {
    let mut h = ResponseHeaderMap::new();
    b.iter(|| {
        h
            .insert("Access-Control-Allow-Credentials", test::black_box("true"))
            .insert("Access-Control-Allow-Headers", test::black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
            .insert("Access-Control-Allow-Origin", test::black_box("https://foo.bar.org"))
            .insert("Access-Control-Max-Age", test::black_box("86400"))
            .insert("Vary", test::black_box("Origin"))
            .insert("Server", test::black_box("ohkami"))
            .insert("Connection", test::black_box("Keep-Alive"))
            .insert("Date", test::black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
            .insert("Alt-Svc", test::black_box("h2=\":433\"; ma=2592000"))
            .insert("Proxy-Authenticate", test::black_box("Basic realm=\"Access to the internal site\""))
            .insert("Referer-Policy", test::black_box("same-origin"))
            .insert("X-Frame-Options", test::black_box("DEBY"));
    });
}




#[bench] fn remove_ohkami(b: &mut test::Bencher) {
    let mut h = ResponseHeaders::init();
    h.set()
        .AccessControlAllowCredentials(test::black_box("true"))
        .AccessControlAllowHeaders(test::black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .AccessControlAllowOrigin(test::black_box("https://foo.bar.org"))
        .AccessControlAllowMethods(test::black_box("POST,GET,OPTIONS,DELETE"))
        .AccessControlMaxAge(test::black_box("86400"))
        .Vary(test::black_box("Origin"))
        .Server(test::black_box("ohkami"))
        .Connection(test::black_box("Keep-Alive"))
        .Date(test::black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .Via(test::black_box("HTTP/1.1 GWA"))
        .AltSvc(test::black_box("h2=\":433\"; ma=2592000;"))
        .ProxyAuthenticate(test::black_box("Basic realm=\"Access to the internal site\""))
        .ReferrerPolicy(test::black_box("same-origin"))
        .XFrameOptions(test::black_box("DENY"));

    b.iter(|| {
        h.set()
            .AccessControlAllowCredentials(test::black_box(None))
            .AccessControlAllowHeaders(test::black_box(None))
            .AccessControlAllowOrigin(test::black_box(None))
            .AccessControlAllowMethods(test::black_box(None))
            .AccessControlMaxAge(test::black_box(None))
            .Vary(test::black_box(None))
            .Server(test::black_box(None))
            .Connection(test::black_box(None))
            .Date(test::black_box(None))
            .Via(test::black_box(None))
            .AltSvc(test::black_box(None))
            .ProxyAuthenticate(test::black_box(None))
            .ReferrerPolicy(test::black_box(None))
            .XFrameOptions(test::black_box(None));
    });
}

#[bench] fn remove_http(b: &mut test::Bencher) {
    let mut h = HeaderMap::new();
    h.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static(test::black_box("true")));
    h.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static(test::black_box("X-Custom-Header,Upgrade-Insecure-Requests")));
    h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static(test::black_box("https://foo.bar.org")));
    h.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static(test::black_box("POST,GET,OPTIONS,DELETE")));
    h.insert(header::ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static(test::black_box("86400")));
    h.insert(header::VARY, HeaderValue::from_static(test::black_box("Origin")));
    h.insert(header::SERVER, HeaderValue::from_static(test::black_box("ohkami")));
    h.insert(header::CONNECTION, HeaderValue::from_static(test::black_box("Keep-Alive")));
    h.insert(header::DATE, HeaderValue::from_static(test::black_box("Wed, 21 Oct 2015 07:28:00 GMT")));
    h.insert(header::VIA, HeaderValue::from_static(test::black_box("HTTP/1.1 GWA")));
    h.insert(header::ALT_SVC, HeaderValue::from_static(test::black_box("h2=\":433\"; ma=2592000;")));
    h.insert(header::PROXY_AUTHENTICATE, HeaderValue::from_static(test::black_box("Basic realm=\"Access to the internal site\"")));
    h.insert(header::REFERRER_POLICY, HeaderValue::from_static("same-origin"));
    h.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

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
    });
}

#[bench] fn remove_fxmap(b: &mut test::Bencher) {
    let mut h = FxMap::new();
    h
        .insert("Access-Control-Allow-Credentials", test::black_box("true"))
        .insert("Access-Control-Allow-Headers", test::black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .insert("Access-Control-Allow-Origin", test::black_box("https://foo.bar.org"))
        .insert("Access-Control-Max-Age", test::black_box("86400"))
        .insert("Vary", test::black_box("Origin"))
        .insert("Server", test::black_box("ohkami"))
        .insert("Connection", test::black_box("Keep-Alive"))
        .insert("Date", test::black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .insert("Alt-Svc", test::black_box("h2=\":433\"; ma=2592000"))
        .insert("Proxy-Authenticate", test::black_box("Basic realm=\"Access to the internal site\""))
        .insert("Referer-Policy", test::black_box("same-origin"))
        .insert("X-Frame-Options", test::black_box("DEBY"));

    b.iter(|| {
        h
            .remove(test::black_box("Access-Control-Allow-Credentials"))
            .remove(test::black_box("Access-Control-Allow-Headers"))
            .remove(test::black_box("Access-Control-Allow-Origin"))
            .remove(test::black_box("Access-Control-Max-Age"))
            .remove(test::black_box("Vary"))
            .remove(test::black_box("Server"))
            .remove(test::black_box("Connection"))
            .remove(test::black_box("Date"))
            .remove(test::black_box("Alt-Svc"))
            .remove(test::black_box("Proxy-Authenticate"))
            .remove(test::black_box("Referer-Policy"))
            .remove(test::black_box("X-Frame-Options"));
    });
}

#[bench] fn remove_headermap(b: &mut test::Bencher) {
    let mut h = ResponseHeaderMap::new();
    h
        .insert("Access-Control-Allow-Credentials", test::black_box("true"))
        .insert("Access-Control-Allow-Headers", test::black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .insert("Access-Control-Allow-Origin", test::black_box("https://foo.bar.org"))
        .insert("Access-Control-Max-Age", test::black_box("86400"))
        .insert("Vary", test::black_box("Origin"))
        .insert("Server", test::black_box("ohkami"))
        .insert("Connection", test::black_box("Keep-Alive"))
        .insert("Date", test::black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .insert("Alt-Svc", test::black_box("h2=\":433\"; ma=2592000"))
        .insert("Proxy-Authenticate", test::black_box("Basic realm=\"Access to the internal site\""))
        .insert("Referer-Policy", test::black_box("same-origin"))
        .insert("X-Frame-Options", test::black_box("DEBY"));

    b.iter(|| {
        h
            .remove(test::black_box("Access-Control-Allow-Credentials"))
            .remove(test::black_box("Access-Control-Allow-Headers"))
            .remove(test::black_box("Access-Control-Allow-Origin"))
            .remove(test::black_box("Access-Control-Max-Age"))
            .remove(test::black_box("Vary"))
            .remove(test::black_box("Server"))
            .remove(test::black_box("Connection"))
            .remove(test::black_box("Date"))
            .remove(test::black_box("Alt-Svc"))
            .remove(test::black_box("Proxy-Authenticate"))
            .remove(test::black_box("Referer-Policy"))
            .remove(test::black_box("X-Frame-Options"));
    });
}




#[bench] fn write_ohkami(b: &mut test::Bencher) {
    let mut h = ResponseHeaders::init();
    h.set()
        .AccessControlAllowCredentials(test::black_box("true"))
        .AccessControlAllowHeaders(test::black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .AccessControlAllowOrigin(test::black_box("https://foo.bar.org"))
        .AccessControlAllowMethods(test::black_box("POST,GET,OPTIONS,DELETE"))
        .AccessControlMaxAge(test::black_box("86400"))
        .Vary(test::black_box("Origin"))
        .Server(test::black_box("ohkami"))
        .Connection(test::black_box("Keep-Alive"))
        .Date(test::black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .Via(test::black_box("HTTP/1.1 GWA"))
        .AltSvc(test::black_box("h2=\":433\"; ma=2592000;"))
        .ProxyAuthenticate(test::black_box("Basic realm=\"Access to the internal site\""))
        .ReferrerPolicy(test::black_box("same-origin"))
        .XFrameOptions(test::black_box("DENY"));

    let mut buf = Vec::new();
    b.iter(|| {
        h.write_ref_to(&mut buf);
    });
}

#[bench] fn write_http(b: &mut test::Bencher) {
    let mut h = HeaderMap::new();
    h.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static(test::black_box("true")));
    h.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static(test::black_box("X-Custom-Header,Upgrade-Insecure-Requests")));
    h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static(test::black_box("https://foo.bar.org")));
    h.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static(test::black_box("POST,GET,OPTIONS,DELETE")));
    h.insert(header::ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static(test::black_box("86400")));
    h.insert(header::VARY, HeaderValue::from_static(test::black_box("Origin")));
    h.insert(header::SERVER, HeaderValue::from_static(test::black_box("ohkami")));
    h.insert(header::CONNECTION, HeaderValue::from_static(test::black_box("Keep-Alive")));
    h.insert(header::DATE, HeaderValue::from_static(test::black_box("Wed, 21 Oct 2015 07:28:00 GMT")));
    h.insert(header::VIA, HeaderValue::from_static(test::black_box("HTTP/1.1 GWA")));
    h.insert(header::ALT_SVC, HeaderValue::from_static(test::black_box("h2=\":433\"; ma=2592000;")));
    h.insert(header::PROXY_AUTHENTICATE, HeaderValue::from_static(test::black_box("Basic realm=\"Access to the internal site\"")));
    h.insert(header::REFERRER_POLICY, HeaderValue::from_static("same-origin"));
    h.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

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
        .insert("Access-Control-Allow-Credentials", test::black_box("true"))
        .insert("Access-Control-Allow-Headers", test::black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .insert("Access-Control-Allow-Origin", test::black_box("https://foo.bar.org"))
        .insert("Access-Control-Max-Age", test::black_box("86400"))
        .insert("Vary", test::black_box("Origin"))
        .insert("Server", test::black_box("ohkami"))
        .insert("Connection", test::black_box("Keep-Alive"))
        .insert("Date", test::black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .insert("Alt-Svc", test::black_box("h2=\":433\"; ma=2592000"))
        .insert("Proxy-Authenticate", test::black_box("Basic realm=\"Access to the internal site\""))
        .insert("Referer-Policy", test::black_box("same-origin"))
        .insert("X-Frame-Options", test::black_box("DEBY"));

    let mut buf = Vec::new();
    b.iter(|| {
        h.write_to(&mut buf);
    });
}

#[bench] fn write_headermap(b: &mut test::Bencher) {
    let mut h = ResponseHeaderMap::new();
    h
        .insert("Access-Control-Allow-Credentials", test::black_box("true"))
        .insert("Access-Control-Allow-Headers", test::black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
        .insert("Access-Control-Allow-Origin", test::black_box("https://foo.bar.org"))
        .insert("Access-Control-Max-Age", test::black_box("86400"))
        .insert("Vary", test::black_box("Origin"))
        .insert("Server", test::black_box("ohkami"))
        .insert("Connection", test::black_box("Keep-Alive"))
        .insert("Date", test::black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
        .insert("Alt-Svc", test::black_box("h2=\":433\"; ma=2592000"))
        .insert("Proxy-Authenticate", test::black_box("Basic realm=\"Access to the internal site\""))
        .insert("Referer-Policy", test::black_box("same-origin"))
        .insert("X-Frame-Options", test::black_box("DEBY"));

    let mut buf = Vec::new();
    b.iter(|| {
        h.write_to(&mut buf);
    });
}
