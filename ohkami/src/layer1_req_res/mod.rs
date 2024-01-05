mod request;  pub use request::*;
mod response; pub use response::*;


#[cfg(test)] #[allow(unused)] mod __ {
    use serde::Serialize;
    use crate::response;

    fn handler_1() -> response::Empty {
        response::Empty::NoContent()
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
    fn handler_2() -> Result<response::JSON<Length>, LengthError> {
        let length = Length::new()?;
        Ok(response::JSON::Created(length))
    }
}
