use crate::header::{append, private::{SameSitePolicy, SetCookie}};
use super::ResponseHeaders;


#[test] fn append_header() {
    let mut h = ResponseHeaders::new();

    h.set().Server(append("X"));
    assert_eq!(h.Server(), Some("X"));
    h.set().Server(append("Y"));
    assert_eq!(h.Server(), Some("X, Y"));
}

#[test] fn append_custom_header() {
    let mut h = ResponseHeaders::new();

    h.set().custom("Custom-Header", append("A"));
    assert_eq!(h.custom("Custom-Header"), Some("A"));
    h.set().custom("Custom-Header", append("B"));
    assert_eq!(h.custom("Custom-Header"), Some("A, B"));
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
