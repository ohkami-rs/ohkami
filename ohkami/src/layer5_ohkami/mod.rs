mod routing;
mod facotory; pub use facotory::{Ohkami};

use crate::{layer3_fang_handler::{Fang, Handlers, IntoFang}, layer4_router::TrieRouter};


/// <br/>
/// 
/// ```ignore
/// async fn main() -> Result<()> {
///     let api_ohkami = Ohkami(())(
///         "/users"
///             .POST(create_user),
///         "/users/:id"
///             .GET(get_user_by_id)
///             .PATCH(update_user),
///     );
/// 
///     // No, no, I'd like to use `log` and `auth` fang...
/// 
///     let api_ohkami = Ohkami((auth, log))(
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
///     Ohkami((log,))(
///         "/hc" .GET(health_check),
///         "/api".by(api_ohkami),
///     ).howl(":3000").await
/// }
/// ```
pub struct Ohkami {
    pub(crate) routes: TrieRouter,
}

impl Ohkami {
    pub(crate) fn new() -> Self {
        Self { routes: TrieRouter::new() }
    }
}

impl Ohkami {
    pub async fn howl(self, address: impl TCPAddress) {
        todo!()
    }
} pub trait TCPAddress {
    fn parse(self) -> String;
} const _: () = {
    impl TCPAddress for u16 {
        fn parse(self) -> String {
            if !(self <= 49151) {panic!("port must be 0 〜 49151")}
            self.to_string()
        }
    }
    impl TCPAddress for &'static str {
        fn parse(self) -> String {
            if self.starts_with(":") {
                "0.0.0.0".to_owned() + self
            } else if self.starts_with("localhost") {
                self.replace("localhost", "127.0.0.1")
            } else {
                self.to_owned()
            }
        }
    }
};
