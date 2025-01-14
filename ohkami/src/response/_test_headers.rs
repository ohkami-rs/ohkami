#![cfg(test)]

use crate::header::{append, SameSitePolicy, SetCookie};
use super::ResponseHeaders;


#[test] fn insert_and_write() {
    let mut h = ResponseHeaders::new();
    h.set().Server("A");
    {
        let mut buf = Vec::new();
        h._write_to(&mut buf);
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "Server: A\r\n\r\n");
    }

    let mut h = ResponseHeaders::new();
    h.set().Server("A").ContentType("application/json");
    h.set().Server("B").ContentLength("100");
    h.set().ContentType("text/html").ContentLength("42");
    {
        let mut buf = Vec::new();
        h._write_to(&mut buf);
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "\
            Server: B\r\n\
            Content-Type: text/html\r\n\
            Content-Length: 42\r\n\
            \r\n\
        ");
    }
}

#[test] fn append_header() {
    let mut h = ResponseHeaders::new();

    h.set().Server(append("X"));
    assert_eq!(h.Server(), Some("X"));
    {
        let mut buf = Vec::new();
        h._write_to(&mut buf);
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "\
            Server: X\r\n\
            \r\n\
        ");
    }

    h.set().Server(append("Y"));
    assert_eq!(h.Server(), Some("X, Y"));
    {
        let mut buf = Vec::new();
        h._write_to(&mut buf);
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "\
            Server: X, Y\r\n\
            \r\n\
        ");
    }
}

#[test] fn append_custom_header() {
    let mut h = ResponseHeaders::new();

    h.set().x("Custom-Header", append("A"));
    assert_eq!(h.get("Custom-Header"), Some("A"));
    {
        let mut buf = Vec::new();
        h._write_to(&mut buf);
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "\
            Custom-Header: A\r\n\
            \r\n\
        ");
    }

    h.set().x("Custom-Header", append("B"));
    assert_eq!(h.get("Custom-Header"), Some("A, B"));
    {
        let mut buf = Vec::new();
        h._write_to(&mut buf);
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "\
            Custom-Header: A, B\r\n\
            \r\n\
        ");
    }
}

#[test] fn parse_setcookie_headers() {
    let mut h = ResponseHeaders::new();
    h.set().SetCookie("id", "42", |d|d.Path("/").SameSiteLax().Secure());
    assert_eq!(h.SetCookie().collect::<Vec<_>>(), [
        SetCookie {
            Cookie:   ("id", "42".into()),
            Expires:  None,
            MaxAge:   None,
            Domain:   None,
            Path:     Some("/".into()),
            Secure:   Some(true),
            HttpOnly: None,
            SameSite: Some(SameSitePolicy::Lax),
        }
    ]);

    let mut h = ResponseHeaders::new();
    h.set()
        .SetCookie("id", "10", |d|d.Path("/").SameSiteLax().Secure())
        .SetCookie("id", "42", |d|d.MaxAge(1280).HttpOnly().Path("/where").SameSiteLax().Secure());
    assert_eq!(h.SetCookie().collect::<Vec<_>>(), [
        SetCookie {
            Cookie:   ("id", "10".into()),
            Expires:  None,
            MaxAge:   None,
            Domain:   None,
            Path:     Some("/".into()),
            Secure:   Some(true),
            HttpOnly: None,
            SameSite: Some(SameSitePolicy::Lax),
        },
        SetCookie {
            Cookie:   ("id", "42".into()),
            Expires:  None,
            MaxAge:   Some(1280),
            Domain:   None,
            Path:     Some("/where".into()),
            Secure:   Some(true),
            HttpOnly: Some(true),
            SameSite: Some(SameSitePolicy::Lax),
        },
    ]);
}
