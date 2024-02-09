mod fangs;
pub mod builtin;

pub use fangs::{FrontFang, BackFang};
pub(crate) use fangs::{Fangs, internal};

use std::{any::{Any, TypeId}, future::Future, sync::Arc};
use crate::{Request, Response};


/// # Fang ー ohkami's middleware system
/// 
/// <br>
/// 
/// *example.rs*
/// ```no_run
/// use ohkami::{Ohkami, Route};
/// use ohkami::{Fang, IntoFang, Response};
/// 
/// struct SetServer;
/// impl IntoFang for SetServer {
///     fn into_fang(self) -> Fang {
///         Fang::back(|res: &mut Response| {
///             res.headers.set()
///                 .Server("ohkami");
///         })
///     }
/// }
/// 
/// #[tokio::main]
/// async fn main() {
///     // Use `with` to give
///     // fangs for your Ohkami...
///     Ohkami::with((SetServer,),
///         "/".GET(|| async {
///             "Hello!"
///         })
///     ).howl("localhost:5000").await
/// }
/// ```
/// 
/// <br>
/// 
/// ---
/// 
/// $ cargo run
/// 
/// ---
/// 
/// $ curl -i http://localhost:5000\
/// HTTP/1.1 200 OK\
/// Content-Length: 6\
/// Content-Type: text/plain; charset=UTF-8\
/// Server: ohkami\
/// \
/// Hello!
#[derive(Clone)]
pub struct Fang {
    id:              TypeId,
    pub(crate) proc: proc::FangProc,
}
impl Fang {
    pub(crate) fn is_front(&self) -> bool {
        matches!(self.proc, proc::FangProc::Front(_))
    }
}
const _: () = {
    impl<'f> PartialEq for &'f Fang {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
};

pub(crate) mod proc {
    use std::{future::Future, pin::Pin, sync::Arc};
    use crate::{Request, Response};

    
    #[derive(Clone)]
    pub enum FangProc {
        Front(FrontFang),
        Back (BackFang),
    }

    #[derive(Clone)]
    pub struct FrontFang(pub(crate) Arc<dyn
        // Fn(&mut Request) -> Pin<
        //     Box<dyn
        //         Future<Output = Result<(), Response>>
        //         + Send + 'static
        //     >
        // > + Send + Sync + 'static
        super::FrontFang
    >);

    #[derive(Clone)]
    pub struct BackFang(pub(crate) Arc<dyn
        Fn(&mut Response, &Request) -> Pin<
            Box<dyn
                Future<Output = Result<(), Response>>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >);
}


/*
const _: () = {
    impl Fang {
        /// Create a *front fang* from the `material`：
        /// 
        /// - `async (&/&mut Request)`
        /// - `async (&/&mut Request) -> Result<(), Response>`
        pub fn front<M>(material: impl IntoFrontFang<M> + 'static) -> Self {
            Self {
                id:   material.type_id(),
                proc: proc::FangProc::Front(material.into_front()),
            }
        }

        /// Create a *back fang* from the `material`：
        /// 
        /// - `async (&/&mut Response)`
        /// - `async (&/&mut Response) -> Result<(), Response>`
        /// - `async (&/&mut Response, &Request)`
        /// - `async (&/&mut Response, &Request) -> Result<(), Response>`
        pub fn back<M>(material: impl IntoBackFang<M> + 'static) -> Self {
            Self {
                id:   material.type_id(),
                proc: proc::FangProc::Back(material.into_back()),
            }
        }
    }

    pub trait IntoFrontFang<M> {fn into_front(self) -> proc::FrontFang;}
    const _: () = {
        impl<F, Fut> IntoFrontFang<fn(&Request)> for F
        where
            F:   Fn(&Request) -> Fut + Sync + Send + 'static,
            Fut: Future<Output = ()> + Send + 'static,
        {
            fn into_front(self) -> proc::FrontFang {
                proc::FrontFang(Arc::new(move |req| {
                    let fut = self(req);
                    Box::pin(async move {fut.await; Ok(())})
                }))
            }
        }
        impl<F, Fut> IntoFrontFang<fn(&mut Request)> for F
        where
            F:   Fn(&mut Request) -> Fut + Sync + Send + 'static,
            Fut: Future<Output = ()> + Send + 'static,
        {
            fn into_front(self) -> proc::FrontFang {
                proc::FrontFang(Arc::new(move |req| {
                    let fut = self(req);
                    Box::pin(async move {fut.await; Ok(())})
                }))
            }
        }

        impl<F, Fut> IntoFrontFang<fn(&Request)->Result<(),Response>> for F
        where
            F:   Fn(&Request)->Fut + Sync + Send + 'static,
            Fut: Future<Output = Result<(), Response>> + Send + 'static,
        {
            fn into_front(self) -> proc::FrontFang {
                proc::FrontFang(Arc::new(move |req| {
                    let fut = self(req);
                    Box::pin(async move {fut.await})
                }))
            }
        }
        impl<'req, F, Fut> IntoFrontFang<fn(&'req mut Request)->Result<(),Response>> for F
        where
            F:   Fn(&'req mut Request)->Fut + Sync + Send + 'static,
            Fut: Future<Output = Result<(), Response>> + Send + 'static,
        {
            fn into_front(self) -> proc::FrontFang {
                proc::FrontFang(Arc::new(move |req| {
                    let fut = self(unsafe {std::mem::transmute(req)});
                    Box::pin(async move {fut.await})
                }))
            }
        }
    };
    
    pub trait IntoBackFang<M> {fn into_back(self) -> proc::BackFang;}
    const _: () = {
        impl<F, Fut> IntoBackFang<fn(&Response)> for F
        where
            F:   Fn(&Response) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static,
        {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, _| {
                    let fut = self(res);
                    Box::pin(async move {fut.await; Ok(())})
                }))
            }
        }
        impl<F, Fut> IntoBackFang<fn(&mut Response)> for F
        where
            F:   Fn(&mut Response) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static,
        {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, _| {
                    let fut = self(res);
                    Box::pin(async move {fut.await; Ok(())})
                }))
            }
        }

        impl<F, Fut> IntoBackFang<fn(&Response, &Request)> for F
        where
            F:   Fn(&Response, &Request) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static,
        {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, req| {
                    let fut = self(res, req);
                    Box::pin(async move {fut.await; Ok(())})
                }))
            }
        }
        impl<F, Fut> IntoBackFang<fn(&mut Response, &Request)> for F
        where
            F:   Fn(&mut Response, &Request) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static,
        {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, req| {
                    let fut = self(res, req);
                    Box::pin(async move {fut.await; Ok(())})
                }))
            }
        }

        impl<F, Fut> IntoBackFang<fn(&Response)->Result<(), Response>> for F
        where
            F:   Fn(&Response) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Result<(), Response>> + Send + 'static,
        {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, _| {
                    let fut = self(res);
                    Box::pin(async move {fut.await})
                }))
            }
        }
        impl<F, Fut> IntoBackFang<fn(&mut Response)->Result<(), Response>> for F
        where
            F:   Fn(&mut Response) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Result<(), Response>> + Send + 'static,
        {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, _| {
                    let fut = self(res);
                    Box::pin(async move {fut.await})
                }))
            }
        }

        impl<F, Fut> IntoBackFang<fn(&Response, &Request)->Result<(), Response>> for F
        where
            F:   Fn(&Response, &Request) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Result<(), Response>> + Send + 'static,
        {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, req| {
                    let fut = self(res, req);
                    Box::pin(async move {fut.await})
                }))
            }
        }
        impl<F, Fut> IntoBackFang<fn(&mut Response, &Request)->Result<(), Response>> for F
        where
            F:   Fn(&mut Response, &Request) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Result<(), Response>> + Send + 'static,
        {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, req| {
                    let fut = self(res, req);
                    Box::pin(async move {fut.await})
                }))
            }
        }
    };
};
*/
