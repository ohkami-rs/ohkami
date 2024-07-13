#![feature(test)]
extern crate test;

use test::black_box;
// fn black_box<T>(t: T) -> T {t}

use http::{HeaderMap, HeaderName, HeaderValue};
use ohkami::__internal__::{RequestHeaders, RequestHeader};
use ohkami_lib::{CowSlice, Slice};
use ohkami_benches::header_hashbrown::{HeaderHashBrown, StandardHeader};
use ohkami_benches::request_headers::{
    fxmap::FxMap,
    headerhashmap::HeaderHashMap,
};


fn input() -> Vec<u8> {
    let input: &[u8; 819] = test::black_box(b"\
        Accept-Language: fr-CH, fr;q=0.9, en;q=0.8, de;q=0.7, *;q=0.5\r\n\
        Authorization: Bearer dummy-authorization-token-sample\r\n\
        Date: Wed, 21 Oct 2015 07:28:00 GMT\r\n\
        Host: localhost:7777\r\n\
        Origin: localhost:3333\r\n\
        Referer: https://developer.mozilla.org/ja/docs/Web/JavaScript\r\n\
        Referrer-Policy: no-referrer\r\n\
        Via: HTTP/1.1 GWA\r\n\
        User-Agent: Mozilla/5.0 (platform; rv:geckoversion) Gecko/geckotrail Firefox/firefoxversion\r\n\
        Transfer-Encoding: identity\r\n\
        Connection: upgrade\r\n\
        Upgrade: a_protocol/1, example ,another_protocol/2.2\r\n\
        Forwarded: for=192.0.2.60; proto=http; by=203.0.113.43\r\n\
        Upgrade-Insecure-Requests: 1\r\n\
        From: webmaster@example.org\r\n\
        X-MyApp-Data: example-custom-header-value\r\n\
        Some-Custom-Header: strawberry\r\n\
        Expect: 100-continue\r\n\
        Cookie: PHPSESSID=298zf09hf012fh2; csrftoken=u32t4o3tb3gg43; _gat=1\r\n\
        Cache-Control: no-cache\r\n\
        \r\n\
    ");

    // let input: &[u8; 485] = test::black_box(b"\
    //     Authorization: Bearer dummy-authorization-token-sample\r\n\
    //     Date: Wed, 21 Oct 2015 07:28:00 GMT\r\n\
    //     Host: localhost:7777\r\n\
    //     Origin: localhost:3333\r\n\
    //     User-Agent: Mozilla/5.0 (platform; rv:geckoversion) Gecko/geckotrail Firefox/firefoxversion\r\n\
    //     Transfer-Encoding: identity\r\n\
    //     Connection: Keep-Alive\r\n\
    //     From: webmaster@example.org\r\n\
    //     X-MyApp-Data: example-custom-header-value\r\n\
    //     Some-Custom-Header: strawberry\r\n\
    //     Cookie: PHPSESSID=298zf09hf012fh2; csrftoken=u32t4o3tb3gg43; _gat=1\r\n\
    //     Cache-Control: no-cache\r\n\
    //     \r\n\
    // ");

    // let input: &[u8; 301] = test::black_box(b"\
    //     Authorization: Bearer dummy-authorization-token-sample\r\n\
    //     Host: localhost:7777\r\n\
    //     Origin: localhost:3333\r\n\
    //     User-Agent: Mozilla/5.0 (platform; rv:geckoversion) Gecko/geckotrail Firefox/firefoxversion\r\n\
    //     From: webmaster@example.org\r\n\
    //     X-MyApp-Data: example-custom-header-value\r\n\
    //     Some-Custom-Header: strawberry\r\n\
    //     \r\n\
    // ");

    // let input: &[u8; 320] = test::black_box(b"\
    //     Authorization: Bearer dummy-authorization-token-sample\r\n\
    //     Host: localhost:7777\r\n\
    //     Origin: localhost:3333\r\n\
    //     User-Agent: Mozilla/5.0 (platform; rv:geckoversion) Gecko/geckotrail Firefox/firefoxversion\r\n\
    //     From: webmaster@example.org\r\n\
    //     Cookie: PHPSESSID=298zf09hf012fh2; csrftoken=u32t4o3tb3gg43; _gat=1\r\n\
    //     Cache-Control: no-cache\r\n\
    //     \r\n\
    // ");

    Vec::from(test::black_box(input))
}


#[bench] fn ohkami_parse(b: &mut test::Bencher) {
    let input = input();

    b.iter(|| {
        let mut r = byte_reader::Reader::new(black_box(input.as_slice()));

        let mut h = RequestHeaders::_init();
        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            r.consume(": ").unwrap();
            if let Some(key) = RequestHeader::from_bytes(key_bytes) {
                h._insert(key, CowSlice::Ref(
                    Slice::from_bytes(r.read_while(|b| b != &b'\r'))
                ));
            } else {
                match key_bytes {
                    b"Cookie" | b"cookie" => (/* skip now */),
                    _ => h._insert_custom(Slice::from_bytes(key_bytes), CowSlice::Ref(
                        Slice::from_bytes(r.read_while(|b| b != &b'\r'))
                    ))
                }
            }
            r.consume("\r\n");
        }
    })
}

#[bench] fn fxmap_parse(b: &mut test::Bencher) {
    let input = input();

    b.iter(|| {
        let mut r = byte_reader::Reader::new(black_box(input.as_slice()));

        let mut h = FxMap::new();
        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            r.consume(": ").unwrap();
            h.insert(
                Slice::from_bytes(key_bytes),
                CowSlice::Ref(Slice::from_bytes(r.read_while(|b| b != &b'\r'))
            ));
            r.consume("\r\n");
        }
    })
}

#[bench] fn headerhashmap_parse(b: &mut test::Bencher) {
    let input = input();

    b.iter(|| {
        let mut r = byte_reader::Reader::new(black_box(input.as_slice()));

        let mut h = HeaderHashMap::default();
        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            r.consume(": ").unwrap();
            h.insert(
                Slice::from_bytes(key_bytes),
                CowSlice::Ref(Slice::from_bytes(r.read_while(|b| b != &b'\r'))
            ));
            r.consume("\r\n");
        }
    })
}

#[bench] fn http_crate_parse(b: &mut test::Bencher) {
    let input = input();

    b.iter(|| {
        let mut r = byte_reader::Reader::new(black_box(input.as_slice()));
        
        let mut h = HeaderMap::new();
        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            r.consume(": ").unwrap();
            h.insert(
                HeaderName::from_bytes(key_bytes).unwrap(),
                HeaderValue::from_bytes(r.read_while(|b| b != &b'\r')).unwrap(),
            );
            r.consume("\r\n");
        }
    })
}

#[bench] fn header_hashbrown_parse(b: &mut test::Bencher) {
    let input = input();

    b.iter(|| {
        let mut r = byte_reader::Reader::new(black_box(input.as_slice()));
        
        let mut h = HeaderHashBrown::<false>::new();
        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            r.consume(": ").unwrap();
            let value_bytes = r.read_while(|b| b != &b'\r');
            r.consume("\r\n");

            match StandardHeader::from_bytes(key_bytes) {
                Some(s) => h.insert_standard_from_reqbytes(s, value_bytes),
                None    => h.insert_from_reqbytes(key_bytes, value_bytes),
            };
        }
    })
}
