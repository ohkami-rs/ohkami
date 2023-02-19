use std::marker::PhantomData;

use serde::Serialize;

use super::{status::OkStatus, header::ResponseHeaders};

pub struct OkResponse<T: Serialize>(
    String,
    PhantomData<fn() -> T>
); impl<T: Serialize> OkResponse<T> {
    #[inline] pub(crate) fn from(
        status: OkStatus,
        additional_headers: &ResponseHeaders,

    ) -> Self {

    }
}

/*

HTTP/1.1 200 OK
Connection: Keep-Alive
Keep-Alive: timeout=5
Content-Type: text/plain; charset=UTF-8
Content-Length: {}
Date: {}
{}
{}
",
                body.len(),
                now(),
                &self.additional_headers.0,
                body

*/
