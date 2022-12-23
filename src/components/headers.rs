use crate::response::format::ResponseFormat;

pub enum AdditionalHeader {
    AccessControlAllowOrigin,
    // ...
}

impl ResponseFormat for AdditionalHeader {
    fn response_format(&self) -> &str {
        match self {
            AdditionalHeader::AccessControlAllowOrigin => "Access-Control-Allow-Origin: ",
        }
    }
}