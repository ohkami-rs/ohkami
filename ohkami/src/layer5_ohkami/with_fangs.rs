use crate::{
    Fang,
    layer0_lib::{Method, Method::*},
};

  
/// ## fang schema
/// 
/// - to make *back fang* : `Fn(Response) -> Response`
/// - to make *front fang* : `Fn(&mut Context, &mut Request)`, or `_ -> Result<(), Response>` for early returning error response
/// 
/// ```
/// use ohkami::prelude::*;
/// use ohkami::{Fang, IntoFang};
/// 
/// struct Log;
/// impl IntoFang for Log {
///     fn bite(self) -> Fang {
///         Fang(|res: Response| {
///             println!("{res:?}");
///             res
///         })
///     }
/// }
/// ```
pub trait IntoFang {
    const METHODS: &'static [Method] = &[GET, PUT, POST, PATCH, DELETE, HEAD, OPTIONS];
    fn bite(self) -> Fang;
}

pub trait Fangs {
    fn collect(self) -> Vec<(&'static [Method], Fang)>;
} macro_rules! impl_for_tuple {
    ( $( $fang:ident ),* ) => {
        impl<$( $fang: IntoFang ),*> Fangs for ( $( $fang,)* ) {
            #[allow(non_snake_case)]
            fn collect(self) -> Vec<(&'static [Method], Fang)> {
                #[allow(unused_mut)]
                let mut fangs = Vec::new();
                let ( $( $fang, )* ) = self;

                $(
                    fangs.push(($fang::METHODS, $fang.bite()));
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
    fn collect(self) -> Vec<(&'static [Method], Fang)> {
        vec![(Self::METHODS, self.bite())]
    }
}
