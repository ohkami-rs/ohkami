use std::any::Any;
use crate::{
    Context, Request, Response,
    layer3_fang_handler::{IntoFang},
};


pub trait Fang {
    fn front(#[allow(unused)] c: &mut Context, req: Request) -> Result<Request, Response> {
        Ok(req)
    }

    fn back(res: Response) -> Response {
        res
    }
}

struct EmptyFang;
impl Fang for EmptyFang {}
fn apply<F: Fang + 'static>(to: &mut Vec<crate::layer3_fang_handler::Fang>) {
    let front = F::front;
    if front.type_id() != EmptyFang::front.type_id() {
        if let Some(proc) = front.into_fang() {
            to.push(proc)
        }
    }

    let back = F::back;
    if back.type_id() != EmptyFang::back.type_id() {
        if let Some(proc) = back.into_fang() {
            to.push(proc)
        }
    }
}

pub trait Fangs {
    fn collect() -> Vec<crate::layer3_fang_handler::Fang>;
} macro_rules! impl_for_tuple {
    ( $( $fang:ident ),* ) => {
        impl<$( $fang:Fang+'static ),*> Fangs for ( $( $fang,)* ) {
            #[allow(non_snake_case)]
            fn collect() -> Vec<crate::layer3_fang_handler::Fang> {
                #[allow(unused_mut)]
                let mut fangs = Vec::new();

                $(
                    apply::<$fang>(&mut fangs);
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
};

// pub trait Fangs<G>: Sized {
//     fn collect(self) -> Vec<Fang>;
// } const _: () = {
//     impl Fangs<()> for () {
//         fn collect(self) -> Vec<Fang> {
//             vec![]
//                 .into_iter()
//                 .filter_map(|of| of)
//                 .collect()
//         }
//     }
// 
//     impl<F1, Args1>
//         Fangs<Args1> for F1
//     where
//         F1: IntoFang<Args1>,
//     {
//         fn collect(self) -> Vec<Fang> {
//             vec![self.into_fang()]
//                 .into_iter()
//                 .filter_map(|of| of)
//                 .collect()
//         }
//     }
// 
//     impl<F1, Args1>
//         Fangs<(Args1,)> for (F1,)
//     where
//         F1: IntoFang<Args1>,
//     {
//         fn collect(self) -> Vec<Fang> {
//             vec![self.0.into_fang()]
//                 .into_iter()
//                 .filter_map(|of| of)
//                 .collect()
//         }
//     }
// 
//     impl<F1, Args1, F2, Args2>
//         Fangs<(Args1, Args2)> for (F1, F2)
//     where
//         F1: IntoFang<Args1>,
//         F2: IntoFang<Args2>,
//     {
//         fn collect(self) -> Vec<Fang> {
//             vec![self.0.into_fang(), self.1.into_fang()]
//                 .into_iter()
//                 .filter_map(|of| of)
//                 .collect()
//         }
//     }
// 
//     impl<F1, Args1, F2, Args2, F3, Args3>
//         Fangs<(Args1, Args2, Args3)> for (F1, F2, F3)
//     where
//         F1: IntoFang<Args1>,
//         F2: IntoFang<Args2>,
//         F3: IntoFang<Args3>,
//     {
//         fn collect(self) -> Vec<Fang> {
//             vec![self.0.into_fang(), self.1.into_fang(), self.2.into_fang()]
//                 .into_iter()
//                 .filter_map(|of| of)
//                 .collect()
//         }
//     }
// 
//     impl<F1, Args1, F2, Args2, F3, Args3, F4, Args4>
//         Fangs<(Args1, Args2, Args3, Args4)> for (F1, F2, F3, F4)
//     where
//         F1: IntoFang<Args1>,
//         F2: IntoFang<Args2>,
//         F3: IntoFang<Args3>,
//         F4: IntoFang<Args4>,
//     {
//         fn collect(self) -> Vec<Fang> {
//             vec![self.0.into_fang(), self.1.into_fang(), self.2.into_fang(), self.3.into_fang()]
//                 .into_iter()
//                 .filter_map(|of| of)
//                 .collect()
//         }
//     }
// 
//     impl<F1, Args1, F2, Args2, F3, Args3, F4, Args4, F5, Args5>
//         Fangs<(Args1, Args2, Args3, Args4, Args5)> for (F1, F2, F3, F4, F5)
//     where
//         F1: IntoFang<Args1>,
//         F2: IntoFang<Args2>,
//         F3: IntoFang<Args3>,
//         F4: IntoFang<Args4>,
//         F5: IntoFang<Args5>,
//     {
//         fn collect(self) -> Vec<Fang> {
//             vec![self.0.into_fang(), self.1.into_fang(), self.2.into_fang(), self.3.into_fang(), self.4.into_fang()]
//                 .into_iter()
//                 .filter_map(|of| of)
//                 .collect()
//         }
//     }
// 
//     impl<F1, Args1, F2, Args2, F3, Args3, F4, Args4, F5, Args5, F6, Args6>
//         Fangs<(Args1, Args2, Args3, Args4, Args5, Args6)> for (F1, F2, F3, F4, F5, F6)
//     where
//         F1: IntoFang<Args1>,
//         F2: IntoFang<Args2>,
//         F3: IntoFang<Args3>,
//         F4: IntoFang<Args4>,
//         F5: IntoFang<Args5>,
//         F6: IntoFang<Args6>,
//     {
//         fn collect(self) -> Vec<Fang> {
//             vec![self.0.into_fang(), self.1.into_fang(), self.2.into_fang(), self.3.into_fang(), self.4.into_fang(), self.5.into_fang()]
//                 .into_iter()
//                 .filter_map(|of| of)
//                 .collect()
//         }
//     }
// };
// 