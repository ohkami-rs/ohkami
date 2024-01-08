mod request;  pub use request::*;
mod response; pub use response::*;


#[cfg(feature="utils")]
#[cfg(test)] #[allow(unused)] mod __ {
    use serde::Serialize;
    use crate::{utils::JSON, http::Status};

    fn handler_1() -> Status {
        Status::NoContent
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
    fn handler_2() -> Result<JSON<Length>, LengthError> {
        let length = Length::new()?;
        Ok(JSON::Created(length))
    }
}
