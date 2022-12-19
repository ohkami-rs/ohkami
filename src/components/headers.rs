use crate::response::format::ResponseFormat;

pub enum Header {
    AccessControlAllowOrigin,
    // ...
}

impl ResponseFormat for Header {
    fn response_format(&self) -> &str {
        match self {
            Header::AccessControlAllowOrigin => "Access-Control-Allow-Origin: ",
        }
    }
}