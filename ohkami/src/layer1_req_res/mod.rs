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
}
