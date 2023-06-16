#![allow(non_snake_case, unused_mut)]

use crate::{
    layer3_fang_handler::{Handlers, ByAnother},
    layer4_router::{TrieRouter},
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
#[allow(non_upper_case_globals)]
pub const Ohkami: init::Ohkami = init::Ohkami;

mod init {
    use crate::layer5_ohkami::with_fangs::Fangs;

    pub struct Ohkami;
    impl Ohkami {
        pub fn with<G>(self, fangs: impl Fangs<G>) -> super::with_fangs::Ohkami {
            super::with_fangs::Ohkami(fangs.collect())
        }
    }
}
mod with_fangs {
    use crate::layer3_fang_handler::Fang;

    pub struct Ohkami(pub(super) Vec<Fang>);
}


trait Rounting {
    fn apply(self, routes: TrieRouter) -> TrieRouter;
} const _: () = {
    impl Rounting for Handlers {
        fn apply(self, routes: TrieRouter) -> TrieRouter {
            routes.register_handlers(self)
        }
    }
    impl Rounting for ByAnother {
        fn apply(self, routes: TrieRouter) -> TrieRouter {
            routes.merge_another(self)
        }
    }
};

macro_rules! init_routing {
    ($( $routing_item:ident ),*) => {
        impl<$($routing_item: Rounting),*> FnOnce<($($routing_item,)*)> for init::Ohkami {
            type Output = super::Ohkami;
            extern "rust-call" fn call_once(self, ($($routing_item,)*): ($($routing_item,)*)) -> Self::Output {
                let mut routes = TrieRouter::new();
                $(
                    routes = $routing_item.apply(routes);
                )*
                super::Ohkami{ routes }
            }
        }
    };
} const _: () = {
    init_routing!();
    init_routing!(R1);
    init_routing!(R1, R2);
    init_routing!(R1, R2, R3);
    init_routing!(R1, R2, R3, R4);
    init_routing!(R1, R2, R3, R4, R5);
    init_routing!(R1, R2, R3, R4, R5, R6);
    init_routing!(R1, R2, R3, R4, R5, R6, R7);
};

macro_rules! with_fangs_routing {
    ($( $routing_item:ident ),*) => {
        impl<$($routing_item: Rounting),*> FnOnce<($($routing_item,)*)> for with_fangs::Ohkami {
            type Output = super::Ohkami;
            extern "rust-call" fn call_once(self, ($($routing_item,)*): ($($routing_item,)*)) -> Self::Output {
                let mut routes = TrieRouter::new();
                for fang in self.0 {
                    routes = routes.apply_fang(fang);
                }
                $(
                    routes = $routing_item.apply(routes);
                )*
                super::Ohkami{ routes }
            }
        }
    };
} const _: () = {
    with_fangs_routing!();
    with_fangs_routing!(R1);
    with_fangs_routing!(R1, R2);
    with_fangs_routing!(R1, R2, R3);
    with_fangs_routing!(R1, R2, R3, R4);
    with_fangs_routing!(R1, R2, R3, R4, R5);
    with_fangs_routing!(R1, R2, R3, R4, R5, R6);
    with_fangs_routing!(R1, R2, R3, R4, R5, R6, R7);
};
