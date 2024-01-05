mod request;  pub use request::*;
mod response; pub use response::*;


#[cfg(test)] #[allow(unused)] mod __ {
    use serde::Serialize;
    use crate::http;

    fn handler_1() -> http::Status {
        crate::http::Status::NoContent
    }

    #[derive(Serialize)]
    struct Length {
        value: usize
    }
    impl Length {
        fn new() -> Result<Self, LengthError> {
            Ok(Self { value: 42 })
        }
    }
    enum LengthError {
        TODO,
    }
    fn handler_2() -> Result<http::JSON<Length>, LengthError> {
        let length = Length::new()?;
        Ok(http::JSON::Created(length))
    }
}
