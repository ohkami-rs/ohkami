mod handlers; pub use handlers::{Handlers, ByAnother, Route};
mod into_handler; pub use into_handler::{IntoHandler};

use std::{pin::Pin, future::Future};
use crate::{
    Context, Request,
    layer0_lib::{List, BufRange},
    layer1_req_res::{Response},
};

pub(crate) const PATH_PARAMS_LIMIT: usize = 2;
pub(crate) type PathParams = List<BufRange, PATH_PARAMS_LIMIT>;


pub struct Handler(
    pub(crate) Box<dyn
        Fn(Request, Context, PathParams) -> Pin<
            Box<dyn
                Future<Output = Response>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
);
