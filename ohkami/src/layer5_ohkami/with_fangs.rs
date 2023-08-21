use crate::{Fang};


pub trait IntoFang {
    fn bite(self) -> Fang;
}

pub trait Fangs {
    fn collect(self) -> Vec<Fang>;
} macro_rules! impl_for_tuple {
    ( $( $fang:ident ),* ) => {
        impl<$( $fang: IntoFang ),*> Fangs for ( $( $fang,)* ) {
            #[allow(non_snake_case)]
            fn collect(self) -> Vec<Fang> {
                #[allow(unused_mut)]
                let mut fangs = Vec::new();
                let ( $( $fang, )* ) = self;

                $(
                    fangs.push($fang.bite());
                )*

                fangs
            }
        }
    };
} const _: () = {
    impl_for_tuple!();
    impl_for_tuple!(F1);
    impl_for_tuple!(F1, F2);
    impl_for_tuple!(F1, F2, F3);
    impl_for_tuple!(F1, F2, F3, F4);
    impl_for_tuple!(F1, F2, F3, F4, F5);
    impl_for_tuple!(F1, F2, F3, F4, F5, F6);
    impl_for_tuple!(F1, F2, F3, F4, F5, F6, F7);
    impl_for_tuple!(F1, F2, F3, F4, F5, F6, F7, F8);
}; impl<F: IntoFang> Fangs for F {
    fn collect(self) -> Vec<Fang> {
        vec![self.bite()]
    }
}
