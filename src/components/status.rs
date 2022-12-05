use crate::response::ResponseFormat;


#[derive(Debug)]
pub(crate) enum Status {
    OK                  = 200,
    Created             = 201,
    BadRequest          = 400,
    Unauthorized        = 401,
    Forbidden           = 403,
    NotFound            = 404,
    InternalServerError = 500,
    NotImplemented      = 501,
}

impl Status {
    pub(crate) fn content_type(&self) -> &'static str {
        match self {
            Self::OK | Self::Created => "application/json",
            _ => "text/plain",
        }
    }
}

impl ResponseFormat for Status {
    fn response_format(&self) -> &'static str {
        match self {
            Self::BadRequest => "400 Bad Request",
            Self::InternalServerError => "500 Internal Server Error",
            Self::NotFound => "404 Not Found",
            Self::Forbidden => "403 Forbidden",
            Self::Unauthorized => "401 Unauthorized",
            Self::NotImplemented => "501 Not Implemented",
            Self::OK => "200 OK",
            Self::Created => "201 Created",
        }
    }
}