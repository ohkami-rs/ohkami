#![cfg(test)]
#![cfg(feature = "__rt_native__")]

use super::ResponseHeaders;
use crate::header::{SameSitePolicy, SetCookie, append};

macro_rules! headers_dump {
    ($dump:literal) => {
        format!(
            $dump,
            NOW = ::ohkami_lib::imf_fixdate(crate::util::unix_timestamp())
        )
    };
}

#[test]
fn insert_and_write() {
    let mut h = ResponseHeaders::new();
    h.set().server("A");
    {
        let mut buf = Vec::new();
        h._write_to(&mut buf);
        assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            headers_dump!(
                "\
            Date: {NOW}\r\n\
            Content-Length: 0\r\n\
            Server: A\r\n\
            \r\n\
        "
            )
        );
    }

    let mut h = ResponseHeaders::new();
    h.set().server("A").content_type("application/json");
    h.set().server("B").content_length("100");
    h.set().content_type("text/html").content_length("42");
    {
        let mut buf = Vec::new();
        h._write_to(&mut buf);
        assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            headers_dump!(
                "\
            Date: {NOW}\r\n\
            Content-Length: 42\r\n\
            Server: B\r\n\
            Content-Type: text/html\r\n\
            \r\n\
        "
            )
        );
    }
}

#[test]
fn append_header() {
    let mut h = ResponseHeaders::new();

    h.set().server(append("X"));
    assert_eq!(h.server(), Some("X"));
    {
        let mut buf = Vec::new();
        h._write_to(&mut buf);
        assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            headers_dump!(
                "\
            Date: {NOW}\r\n\
            Content-Length: 0\r\n\
            Server: X\r\n\
            \r\n\
        "
            )
        );
    }

    h.set().server(append("Y"));
    assert_eq!(h.server(), Some("X, Y"));
    {
        let mut buf = Vec::new();
        h._write_to(&mut buf);
        assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            headers_dump!(
                "\
            Date: {NOW}\r\n\
            Content-Length: 0\r\n\
            Server: X, Y\r\n\
            \r\n\
        "
            )
        );
    }
}

#[test]
fn append_custom_header() {
    let mut h = ResponseHeaders::new();

    h.set().x("Custom-Header", append("A"));
    assert_eq!(h.get("Custom-Header"), Some("A"));
    {
        let mut buf = Vec::new();
        h._write_to(&mut buf);
        assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            headers_dump!(
                "\
            Date: {NOW}\r\n\
            Content-Length: 0\r\n\
            Custom-Header: A\r\n\
            \r\n\
        "
            )
        );
    }

    h.set().x("Custom-Header", append("B"));
    assert_eq!(h.get("Custom-Header"), Some("A, B"));
    {
        let mut buf = Vec::new();
        h._write_to(&mut buf);
        assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            headers_dump!(
                "\
            Date: {NOW}\r\n\
            Content-Length: 0\r\n\
            Custom-Header: A, B\r\n\
            \r\n\
        "
            )
        );
    }
}

#[test]
fn parse_setcookie_headers() {
    let mut h = ResponseHeaders::new();
    h.set()
        .set_cookie("id", "42", |d| d.path("/").same_site_lax().secure(true));
    assert_eq!(
        h.set_cookie().collect::<Vec<_>>(),
        [SetCookie {
            cookie: ("id", "42".into()),
            expires: None,
            max_age: None,
            domain: None,
            path: Some("/".into()),
            secure: Some(true),
            http_only: None,
            same_site: Some(SameSitePolicy::Lax),
        }]
    );

    let mut h = ResponseHeaders::new();
    h.set()
        .set_cookie("id", "10", |d| d.path("/").same_site_lax().secure(true))
        .set_cookie("id", "42", |d| {
            d.max_age(1280)
                .http_only()
                .path("/where")
                .same_site_lax()
                .secure(true)
        });
    assert_eq!(
        h.set_cookie().collect::<Vec<_>>(),
        [
            SetCookie {
                cookie: ("id", "10".into()),
                expires: None,
                max_age: None,
                domain: None,
                path: Some("/".into()),
                secure: Some(true),
                http_only: None,
                same_site: Some(SameSitePolicy::Lax),
            },
            SetCookie {
                cookie: ("id", "42".into()),
                expires: None,
                max_age: Some(1280),
                domain: None,
                path: Some("/where".into()),
                secure: Some(true),
                http_only: Some(true),
                same_site: Some(SameSitePolicy::Lax),
            },
        ]
    );
}
