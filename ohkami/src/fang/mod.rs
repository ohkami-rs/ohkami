use std::{pin::Pin, future::Future};
use crate::{router::route::FangRoute, response::err::ErrResponse, context::Context};

pub struct Fangs(Vec<Fang>);
struct Fang {
    route: FangRoute,
    proc:  FangProc,
}
type FangProc =
    Box<dyn
        Fn(Context, ) -> Pin<
            Box<
                dyn Future<Output=Result<(), ErrResponse>> + Send
            >
        > + Send + Sync
    >
;
