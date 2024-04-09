#![feature(test)]
extern crate test;

use ohkami::__internal__::{RequestHeaders, RequestHeader};
use ohkami_lib::{CowSlice, Slice};

use ohkami_benches::request_headers::{
    fxmap::FxMap,
    heap_ohkami_headers::{
        HeapOhkamiHeaders, Header as HeapOhkamiHeader
    },
};


const INPUT: &[u8] = b"\
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
\r\n";


#[bench] fn ohkami_parse(b: &mut test::Bencher) {
    b.iter(|| {
        let mut r = byte_reader::Reader::new(test::black_box(INPUT));

        let mut h = RequestHeaders::__init__();
        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            r.consume(": ").unwrap();
            if let Some(key) = RequestHeader::from_bytes(key_bytes) {
                h._insert(key, CowSlice::Ref(unsafe {
                    Slice::from_bytes(r.read_while(|b| b != &b'\r'))
                }));
            } else {
                let key = CowSlice::Ref(unsafe {Slice::from_bytes(key_bytes)});
                h._insert_custom(key, CowSlice::Ref(unsafe {
                    Slice::from_bytes(r.read_while(|b| b != &b'\r'))
                }));
            }
            r.consume("\r\n");
        }
    })
}

#[bench] fn heap_ohkami_parse(b: &mut test::Bencher) {
    b.iter(|| {
        let mut r = byte_reader::Reader::new(test::black_box(INPUT));

        let mut h = HeapOhkamiHeaders::new();
        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            r.consume(": ").unwrap();
            if let Some(key) = HeapOhkamiHeader::from_bytes(key_bytes) {
                h.insert(key, CowSlice::Ref(unsafe {
                    Slice::from_bytes(r.read_while(|b| b != &b'\r'))
                }));
            } else {
                let key = unsafe {Slice::from_bytes(key_bytes)};
                h.insert_custom(key, CowSlice::Ref(unsafe {
                    Slice::from_bytes(r.read_while(|b| b != &b'\r'))
                }));
            }
            r.consume("\r\n");
        }
    })
}

#[bench] fn fxmap_parse(b: &mut test::Bencher) {
    b.iter(|| {
        let mut r = byte_reader::Reader::new(test::black_box(INPUT));

        let mut h = FxMap::new();
        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            r.consume(": ").unwrap();
            h.insert(
                unsafe {Slice::from_bytes(key_bytes)},
                CowSlice::Ref(unsafe {Slice::from_bytes(r.read_while(|b| b != &b'\r'))}
            ));
            r.consume("\r\n");
        }
    })
}
