mod with_fangs;
mod build; pub use build::{Ohkami};
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
///     // I'd like to use `auth` and `log` fang...
///     
///     let api_ohkami = Ohkami.with((auth, log))(
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
///     Ohkami.with((log))(
///         "/hc" .GET(health_check),
///         "/api".by(api_ohkami),
///     ).howl(3000).await
/// }
/// ```
pub struct Ohkami {
    pub(crate) routes: TrieRouter,
}
