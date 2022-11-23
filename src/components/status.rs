use crate::response::ResponseFormat;


#[derive(Debug)]
pub(crate) enum Status {
    SetUpError,
    OK                  = 200,
    Created             = 201,
    BadRequest          = 400,
    NotFound            = 404,
    InternalServerError = 500,
    NotImplemented      = 501,
}

impl Status {
    pub(crate) fn content_type(&self) -> &'static str {
        match self {
            Self::OK => "application/json",
            _ => "text/plain",
        }
    }
}

impl ResponseFormat for Status {
    fn response_format(&self) -> &'static str {
        match self { Self::SetUpError => unreachable!(),
            Self::BadRequest => "400 BadRequest",
            Self::InternalServerError => "500 InternalServerError",
            Self::NotFound => "404 NotFound",
            Self::NotImplemented => "501 NotImplemented",
            Self::OK => "200 OK",
            Self::Created => "201 Created",
        }
    }
}