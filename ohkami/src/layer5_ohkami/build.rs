#![allow(non_snake_case, unused_mut)]

use crate::{
    layer3_fang_handler::{Handlers, ByAnother},
    layer4_router::{TrieRouter},
};


trait RoutingItem {
    fn apply(self, routes: TrieRouter) -> TrieRouter;
} const _: () = {
    impl RoutingItem for Handlers {
        fn apply(self, routes: TrieRouter) -> TrieRouter {
            routes.register_handlers(self)
        }
    }
    impl RoutingItem for ByAnother {
        fn apply(self, routes: TrieRouter) -> TrieRouter {
            routes.merge_another(self)
        }
    }
};

pub trait Routes {
    fn apply(self, routes: TrieRouter) -> TrieRouter;
} macro_rules! impl_for_tuple {
    ( $( $item:ident ),* ) => {
        impl<$( $item: RoutingItem ),*> Routes for ( $($item,)* ) {
            fn apply(self, mut routes: TrieRouter) -> TrieRouter {
                let ( $( $item, )* ) = self;
                $(
                    routes = <$item as RoutingItem>::apply($item, routes);
                )*
                routes
            }
        }
    };
} const _: () = {
    impl_for_tuple!();
    impl_for_tuple!(R1);
    impl_for_tuple!(R1, R2);
    impl_for_tuple!(R1, R2, R3);
    impl_for_tuple!(R1, R2, R3, R4);
    impl_for_tuple!(R1, R2, R3, R4, R5);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8);
};

// macro_rules! build_routing {
//     ($( $routing_item:ident ),*) => {
//         impl<$($routing_item: Rounting),*> FnOnce<($($routing_item,)*)> for Ohkami {
//             type Output = Ohkami;
//             extern "rust-call" fn call_once(mut self, ($($routing_item,)*): ($($routing_item,)*)) -> Self::Output {
//                 $(
//                     self.routes = $routing_item.apply(self.routes);
//                 )*
//                 self
//             }
//         }
//     };
// } const _: () = {
//     build_routing!();
//     build_routing!(R1);
//     build_routing!(R1, R2);
//     build_routing!(R1, R2, R3);
//     build_routing!(R1, R2, R3, R4);
//     build_routing!(R1, R2, R3, R4, R5);
//     build_routing!(R1, R2, R3, R4, R5, R6);
//     build_routing!(R1, R2, R3, R4, R5, R6, R7);
// };
// 