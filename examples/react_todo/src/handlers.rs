pub(crate) mod user;

use ohkami::{
    result::Result,
    context::Context,
    response::Response,
};

pub(crate) async fn root(c: Context) -> Result<Response> {
    c.OK("Hello, World!")
}