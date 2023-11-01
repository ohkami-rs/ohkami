#[derive(PartialEq)]
pub enum Status {
    SwitchingProtocols,

    OK,
    Created,
    NoContent,

    MovedPermanently,
    Found,

    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,

    InternalServerError,
    NotImplemented,
} impl Status {
    #[inline(always)] pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::SwitchingProtocols  => "101 Switching Protocols",

            Self::OK                  => "200 OK",
            Self::Created             => "201 Created",
            Self::NoContent           => "204 No Content",

            Self::MovedPermanently    => "301 Moved Permanently",
            Self::Found               => "302 Found",

            Self::BadRequest          => "400 Bad Request",
            Self::Unauthorized        => "401 Unauthorized",
            Self::Forbidden           => "403 Forbidden",
            Self::NotFound            => "404 Not Found",
            Self::InternalServerError => "500 Internal Server Error",
            Self::NotImplemented      => "501 Not Implemented",
        }
    }
    #[inline(always)] pub(crate) const fn as_bytes(&self) -> &'static [u8] {
        self.as_str().as_bytes()
    }
} const _: () = {
    impl std::fmt::Debug for Status {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.as_str())
        }
    }
};
