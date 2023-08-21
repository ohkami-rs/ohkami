mod with_fangs; pub use with_fangs::{IntoFang};
mod build;
mod howl;

use crate::{layer4_router::TrieRouter};


/// <br/>
/// 
/// ```ignore
/// async fn main() {
///     let api_ohkami = Ohkami::new()(
///         "/users"
///             .POST(create_user),
///         "/users/:id"
///             .GET(get_user_by_id)
///             .PATCH(update_user),
///     );
/// 
///     // I'd like to use `Auth` and `Log` fang...
///     
///     let api_ohkami = Ohkami::with((Auth, Log), (
///         "/users"
///             .POST(create_user),
///         "/users/:id"
///             .GET(get_user_by_id).
///             .PATCH(update_user),
///     );
/// 
///     // And, actually, `Log` fang of api_ohkami is duplicated with
///     // that of the root ohkami below, but it's no problem
///     // because they are merged internally.
/// 
///     Ohkami::with(Log, (
///         "/hc" .GET(health_check),
///         "/api".By(api_ohkami),
///     ).howl(3000).await
/// }
/// ```
/// 
/// <br/>
/// 
/// ## fang schema
/// - front: `(&mut Context, Request) -> Result<Request, Response>`
/// - back:  `(Response) -> Response`
/// 
/// ## handler schema
/// - async (`Context`) -> `Response`
/// - async (`Context`, {path_params}) -> `Response`
/// - async (`Context`, some {impl `FromRequest`}s) -> `Response`
/// - async (`Context`, {path_params}, some {impl `FromRequest`}s) -> `Response`
/// 
/// path_params :
///   - `String`
///   - `u8` ~ `u128`, `usize`
///   - and tuple of them
pub struct Ohkami {
    pub(crate) routes: TrieRouter,

    /// apply just before merged to another or called `howl`
    pub(crate) fangs:  Vec<crate::layer3_fang_handler::Fang>,
}

impl Ohkami {
    pub fn new(routes: impl build::Routes) -> Self {
        Self {
            routes: routes.apply(TrieRouter::new()),
            fangs:  Vec::new(),
        }
    }

    pub fn with(fangs: impl with_fangs::Fangs, routes: impl build::Routes) -> Self {
        Self {
            routes: routes.apply(TrieRouter::new()),
            fangs:  fangs.collect(),
        }
    }
}

impl Ohkami {
    pub(crate) fn into_router(self) -> TrieRouter {
        let mut router = self.routes;
        for fang in self.fangs {
            router = router.apply_fang(fang)
        }
        router
    }
}
