use crate::{Response, Request, IntoResponse, Method::{self, *}};

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
use {
    crate::fang::Fang,
    std::{future::Future, pin::Pin},
};


/// Represents "can be used as a front fang", e.g. executed before `req` is passed to a handler.\
/// `METHODS` const parameter enable to filter the target request of this fang by request method
/// (default: any methods).
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
/// 
/// struct LogRequest;
/// impl FrontFang for LogRequest {
///     type Error = std::convert::Infallible;
///     async fn bite(&self, req: &mut Request) -> Result<(), Self::Error> {
///         println!("{req:?}");
///         Ok(())
///     }
/// }
/// ```
pub trait FrontFang {
    const METHODS: &'static [Method] = &[GET, PUT, POST, PATCH, DELETE, HEAD, OPTIONS];

    /// If `bite` never fails, `std::convert::Infallible` is recommended.
    type Error: IntoResponse;

    #[must_use]
    #[allow(clippy::type_complexity)]
    fn bite(&self, req: &mut Request) -> impl ::std::future::Future<Output = Result<(), Self::Error>> + Send;
}

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
pub(crate) trait FrontFangCaller: Send + Sync {
    fn call<'c>(&'c self, req: &'c mut Request) -> Pin<Box<dyn Future<Output = Result<(), Response>> + Send + 'c>>
    where Self: Sync + 'c;
}
#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
impl<FF: FrontFang + Send + Sync> FrontFangCaller for FF {
    #[inline(always)] fn call<'c>(&'c self, req: &'c mut Request) -> Pin<Box<dyn Future<Output = Result<(), Response>> + Send + 'c>>
    where Self: Sync + 'c {
        Box::pin(async {
            self.bite(req).await
                .map_err(IntoResponse::into_response)
        })
    }
}


/// Represents "can be used as a back fang", e.g. executed after a handler generates `res`.\
/// `METHODS` const parameter enable to filter the target request of this fang by request method
/// (default: any methods).
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
/// 
/// struct LogResponse;
/// impl BackFang for LogResponse {
///     type Error = std::convert::Infallible;
///     async fn bite(&self, res: &mut Response, _req: &Request) -> Result<(), Self::Error> {
///         println!("{res:?}");
///         Ok(())
///     }
/// }
/// ```
pub trait BackFang {
    const METHODS: &'static [Method] = &[GET, PUT, POST, PATCH, DELETE, HEAD, OPTIONS];

    /// If `bite` never fails, `std::convert::Infallible` is recommended.
    type Error: IntoResponse;

    #[must_use]
    #[allow(clippy::type_complexity)]
    fn bite(&self, res: &mut Response, _req: &Request) -> impl ::std::future::Future<Output = Result<(), Self::Error>> + Send;
}

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
pub(crate) trait BackFangCaller: Send + Sync {
    fn call<'c>(&'c self, res: &'c mut Response, _req: &'c Request) -> Pin<Box<dyn Future<Output = Result<(), Response>> + Send + 'c>>
    where Self: Sync + 'c;
}
#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
impl<BF: BackFang + Send + Sync> BackFangCaller for BF {
    #[inline(always)] fn call<'c>(&'c self, res: &'c mut Response, _req: &'c Request) -> Pin<Box<dyn Future<Output = Result<(), Response>> + Send + 'c>>
    where Self: Sync + 'c {
        Box::pin(async {
            self.bite(res, _req).await
                .map_err(IntoResponse::into_response)
        })
    }
}


#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
pub(crate) mod internal {
    use crate::{Method, Method::*};
    use super::super::Fang;
    use std::any::Any;

    use {
        super::super::proc::{FangProc, FrontFang, BackFang},
        std::sync::Arc,
    };
    
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
                proc: FangProc::Front(FrontFang(Arc::new(self))),
            }
        }
    }
    
    pub struct Back;
    impl<BF: super::BackFang + Send + Sync + 'static> IntoFang<Back> for BF {
        const METHODS: &'static [Method] = BF::METHODS;
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Back(BackFang(Arc::new(self))),
            }
        }
    }

    pub struct SpecialBuiltin;
    impl IntoFang<SpecialBuiltin> for crate::builtin::fang::Timeout {
        const METHODS: &'static [Method] = &[GET, PUT, POST, PATCH, DELETE, OPTIONS, HEAD];
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
                proc: FangProc::Timeout(self),
            }
        }
    }
}


#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
pub trait Fangs<T> {
    fn collect(self) -> Vec<(&'static [Method], Fang)>;
}

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
const _: () = {
    macro_rules! impl_for_tuple {
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
    }

    impl_for_tuple!();
    impl_for_tuple!(F1:T1);
    impl_for_tuple!(F1:T1, F2:T2);
    impl_for_tuple!(F1:T1, F2:T2, F3:T3);
    impl_for_tuple!(F1:T1, F2:T2, F3:T3, F4:T4);
    impl_for_tuple!(F1:T1, F2:T2, F3:T3, F4:T4, F5:T5);
    impl_for_tuple!(F1:T1, F2:T2, F3:T3, F4:T4, F5:T5, F6:T6);
    impl_for_tuple!(F1:T1, F2:T2, F3:T3, F4:T4, F5:T5, F6:T6, F7:T7);
    impl_for_tuple!(F1:T1, F2:T2, F3:T3, F4:T4, F5:T5, F6:T6, F7:T7, F8:T8);

    impl<T, F: internal::IntoFang<T>> Fangs<T> for F {
        fn collect(self) -> Vec<(&'static [Method], Fang)> {
            vec![(Self::METHODS, self.into_fang())]
        }
    }
};
