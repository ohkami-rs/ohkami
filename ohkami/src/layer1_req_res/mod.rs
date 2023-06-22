mod request; pub use request::*;
mod response; pub use response::*;


#[cfg(test)] #[allow(unused)] mod __ {
    use serde::Serialize;
    use crate::Context;
    use super::Response;

    fn handler_1(mut c: Context) -> Response<()> {
        c.NoContent()
    }

    #[derive(Serialize)]
    struct Length {
        value: usize
    }
    impl Length {
        fn new() -> Result<Self, std::io::Error> {
            Ok(Self { value: 42 })
        }
    }
    fn handler_2(mut c: Context) -> Response<Length> {
        let length = Length::new()
            .map_err(|_| c.InternalError().Text("got error in I/O"))?;

        c.Created(length)
    }


    #[test]
    fn check_response_headers_2() {
        use super::response::ResponseHeaders;

        let mut headers = ResponseHeaders::new();
        headers
            .Server("ohkami")
            .CacheControl("no-store");

        let __now__ = crate::layer0_lib::now();
        assert_eq!(headers.to_string(), format!("\
            Connection: Keep-Alive\r\n\
            Keep-Alive: timout=5\r\n\
            Date: {__now__}\r\n\
            Cache-Control: no-store\r\n\
            Access-Control-Allow-Origin: https://kanarusblog.software\r\n\
            Server: ohkami\r\n\
            \r\n\
        "))
    }
}
