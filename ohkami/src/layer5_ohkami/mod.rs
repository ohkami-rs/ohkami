mod with_fangs;
mod routing;
mod howl;

use crate::{
    layer4_router::TrieRouter,
};


/// <br/>
/// 
/// ```ignore
/// async fn main() -> Result<()> {
///     let api_ohkami = Ohkami(
///         "/users"
///             .POST(create_user),
///         "/users/:id"
///             .GET(get_user_by_id)
///             .PATCH(update_user),
///     );
/// 
///     // I'd like to use `log` and `auth` fang...
/// 
///     let api_ohkami = Ohkami(auth, log, // <----
///         "/users"
///             .POST(create_user),
///         "/users/:id"
///             .GET(get_user_by_id)
///             .PATCH(update_user),
///     );
/// 
///     // (Actually, this `log` fang of api_ohkami is duplicated with
///     // `log` fang of the root ohkami below, but there's no problem
///     // because they are merged internally.)
/// 
///     Ohkami(log,
///         "/hc" .GET(health_check),
///         "/api".by(api_ohkami),
///     ).howl(3000).await
/// }
/// ```
pub struct Ohkami {
    pub(crate) routes: TrieRouter,
}

impl Ohkami {
    pub fn new() -> Self {
        Self { routes: TrieRouter::new() }
    }

    pub fn with<G>(fangs: impl with_fangs::Fangs<G>) -> Self {
        Self { routes: fangs.apply(TrieRouter::new()) }
    }
}
