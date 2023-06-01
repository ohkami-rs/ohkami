pub(crate) enum Status {
    OK                  = 200,
    Created             = 201,
    NoContent           = 204,
    BadRequest          = 400,
    Unauthorized        = 401,
    Forbidden           = 403,
    NotFound            = 404,
    InternalServerError = 500,
    NotImplemented      = 501,
} impl Status {
    #[inline(always)] pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::OK                  => "200 OK",
            Self::Created             => "201 Created",
            Self::NoContent           => "204 No Content",
            Self::BadRequest          => "400 Bad Request",
            Self::Unauthorized        => "401 Unauthorized",
            Self::Forbidden           => "403 Forbidden",
            Self::NotFound            => "404 Not Found",
            Self::InternalServerError => "500 Internal Server Error",
            Self::NotImplemented      => "501 Not Implemented",
        }
    }

    #[inline(always)] pub(crate) const fn is_error(&self) -> bool {
        match self {
            Self::OK | Self::Created | Self::NoContent => false,
            _ => true
        }
    }
}
