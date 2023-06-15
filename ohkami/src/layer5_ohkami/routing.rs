#![allow(non_snake_case, unused_mut)]

use super::Ohkami;
use crate::{layer3_fang_handler::{Handlers, ByAnother}, layer4_router::TrieRouter};


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

macro_rules! routing {
    ($( $routing_item:ident ),*) => {
        impl<$($routing_item: Rounting),*> FnOnce<($($routing_item,)*)> for Ohkami {
            type Output = Ohkami;
            extern "rust-call" fn call_once(self, ($($routing_item,)*): ($($routing_item,)*)) -> Self::Output {
                let mut routes = self.routes;
                $(
                    routes = $routing_item.apply(routes);
                )*
                Ohkami{ routes }
            }
        }
    };
} const _: () = {
    routing!();
    routing!(R1);
    routing!(R1, R2);
    routing!(R1, R2, R3);
    routing!(R1, R2, R3, R4);
    routing!(R1, R2, R3, R4, R5);
    routing!(R1, R2, R3, R4, R5, R6);
    routing!(R1, R2, R3, R4, R5, R6, R7);
};
