use crate::{Fang, Response, Request, Method::{self, *}};


/// Represents "can be used as a front fang".
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
/// 
/// struct LogRequest;
/// impl FrontFang for LogRequest {
///     async fn bite(self, req: &mut Request) -> Result<(), Response> {
///         println!("{req:?}");
///     }
/// }
/// ```
pub trait FrontFang {
    const METHODS: &'static [Method] = &[GET, PUT, POST, PATCH, DELETE, HEAD, OPTIONS];

    fn bite(&self, req: &mut Request) -> impl ::std::future::Future<Output = Result<(), Response>> + Send;
}

/// Represents "can be used as a back fang".
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
/// 
/// struct LogResponse;
/// impl FrontFang for LogResponse {
///     async fn bite(&self, res: &mut Response) -> Result<(), Response> {
///         println!("{res:?}");
///     }
/// }
/// ```
pub trait BackFang {
    const METHODS: &'static [Method] = &[GET, PUT, POST, PATCH, DELETE, HEAD, OPTIONS];

    fn bite(&self, res: &mut Response, req: &Request) -> impl ::std::future::Future<Output = Result<(), Response>> + Send;
}


pub(crate) mod internal {
    use std::{any::Any, sync::Arc};
    use crate::{Method, Fang};
    use super::super::proc::{FangProc, FrontFang, BackFang};
    
    pub trait IntoFang<T> {
        const METHODS: &'static [Method];
        fn into_fang(self) -> Fang;
    }
    
    pub struct Front;
    impl<FF: super::FrontFang + Send + Sync + 'static> IntoFang<Front> for FF {
        const METHODS: &'static [Method] = FF::METHODS;

        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(|req| {
                    // let fut = self.bite(req);
                    // Box::pin(fut)

                    todo!()
                }))),
            }
        }
    }
    
    pub struct Back;
    impl<BF: super::BackFang + 'static> IntoFang<Back> for BF {
        const METHODS: &'static [Method] = BF::METHODS;

        fn into_fang(self) -> Fang {
            todo!()
        }
    }
}


pub trait Fangs<T> {
    fn collect(self) -> Vec<(&'static [Method], Fang)>;
} macro_rules! impl_for_tuple {
    () => {
        impl Fangs<()> for () {
            fn collect(self) -> Vec<(&'static [Method], Fang)> {
                Vec::new()
            }
        }
    };
    ( $( $f:ident : $t:ident ),+ ) => {
        impl<$( $t, $f: internal::IntoFang<$t> ),+> Fangs<( $( $t, )+ )> for ( $( $f,)+ ) {
            #[allow(non_snake_case)]
            fn collect(self) -> Vec<(&'static [Method], Fang)> {
                let mut fangs = Vec::new();
                let ( $( $f, )+ ) = self;

                $(
                    fangs.push(($f::METHODS, $f.into_fang()));
                )+

                fangs
            }
        }
    };
} const _: () = {
    impl_for_tuple!();
    impl_for_tuple!(F1:T1);
    impl_for_tuple!(F1:T1, F2:T2);
    impl_for_tuple!(F1:T1, F2:T2, F3:T3);
    impl_for_tuple!(F1:T1, F2:T2, F3:T3, F4:T4);
    impl_for_tuple!(F1:T1, F2:T2, F3:T3, F4:T4, F5:T5);
    impl_for_tuple!(F1:T1, F2:T2, F3:T3, F4:T4, F5:T5, F6:T6);
    impl_for_tuple!(F1:T1, F2:T2, F3:T3, F4:T4, F5:T5, F6:T6, F7:T7);
    impl_for_tuple!(F1:T1, F2:T2, F3:T3, F4:T4, F5:T5, F6:T6, F7:T7, F8:T8);
}; impl<T, F: internal::IntoFang<T>> Fangs<T> for F {
    fn collect(self) -> Vec<(&'static [Method], Fang)> {
        vec![(Self::METHODS, self.into_fang())]
    }
}
