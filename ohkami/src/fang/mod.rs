pub mod builtin;

use std::{any::{Any, TypeId}, sync::Arc};
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
    use std::sync::Arc;
    use crate::{Request, Response};

    
    #[derive(Clone)]
    pub enum FangProc {
        Front(FrontFang),
        Back (BackFang),
    }

    #[derive(Clone)]
    pub struct FrontFang(pub(crate) Arc<dyn
        Fn(&mut Request) -> Result<(), Response>
        + Send
        + Sync
        + 'static
    >);
    #[derive(Clone)]
    pub struct BackFang(pub(crate) Arc<dyn
        Fn(&mut Response, &Request) -> Result<(), Response>
        + Send
        + Sync
        + 'static
    >);
}


const _: () = {
    impl Fang {
        /// Create a *front fang* from the `material`：
        /// 
        /// - `Fn(&/&mut Request)`
        /// - `Fn(&/&mut Request) -> Result<(), Response>`
        pub fn front<M>(material: impl IntoFrontFang<M> + 'static) -> Self {
            Self {
                id:   material.type_id(),
                proc: proc::FangProc::Front(material.into_front()),
            }
        }

        /// Create a *back fang* from the `material`：
        /// 
        /// - `Fn(&/&mut Response)`
        /// - `Fn(&/&mut Response) -> Result<(), Response>`
        /// - `Fn(&/&mut Response, &Request)`
        /// - `Fn(&/&mut Response, &Request) -> Result<(), Response>`
        pub fn back<M>(material: impl IntoBackFang<M> + 'static) -> Self {
            Self {
                id:   material.type_id(),
                proc: proc::FangProc::Back(material.into_back()),
            }
        }
    }

    pub trait IntoFrontFang<M> {fn into_front(self) -> proc::FrontFang;}
    const _: () = {
        impl<F: Fn(&Request) + Sync + Send + 'static>
        IntoFrontFang<fn(&Request)> for F {
            fn into_front(self) -> proc::FrontFang {
                proc::FrontFang(Arc::new(move |req| {
                    self(req);
                    Ok(())
                }))
            }
        }
        impl<F: Fn(&mut Request) + Sync + Send + 'static>
        IntoFrontFang<fn(&mut Request)> for F {
            fn into_front(self) -> proc::FrontFang {
                proc::FrontFang(Arc::new(move |req| {
                    self(req);
                    Ok(())
                }))
            }
        }

        impl<F: Fn(&Request)->Result<(),Response> + Sync + Send + 'static>
        IntoFrontFang<fn(&Request)->Result<(),Response>> for F {
            fn into_front(self) -> proc::FrontFang {
                proc::FrontFang(Arc::new(move |req| {
                    self(req)
                }))
            }
        }
        impl<F: Fn(&mut Request)->Result<(),Response> + Sync + Send + 'static>
        IntoFrontFang<fn(&mut Request)->Result<(),Response>> for F {
            fn into_front(self) -> proc::FrontFang {
                proc::FrontFang(Arc::new(
                    self
                ))
            }
        }
    };
    
    pub trait IntoBackFang<M> {fn into_back(self) -> proc::BackFang;}
    const _: () = {
        impl<F: Fn(&Response) + Send + Sync + 'static>
        IntoBackFang<fn(&Response)> for F {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, _| {
                    self(res);
                    Ok(())
                }))
            }
        }
        impl<F: Fn(&mut Response) + Send + Sync + 'static>
        IntoBackFang<fn(&mut Response)> for F {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, _| {
                    self(res);
                    Ok(())
                }))
            }
        }

        impl<F: Fn(&Response, &Request) + Send + Sync + 'static>
        IntoBackFang<fn(&Response, &Request)> for F {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, req| {
                    self(res, req);
                    Ok(())
                }))
            }
        }
        impl<F: Fn(&mut Response, &Request) + Send + Sync + 'static>
        IntoBackFang<fn(&mut Response, &Request)> for F {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, req| {
                    self(res, req);
                    Ok(())
                }))
            }
        }

        impl<F: Fn(&Response)->Result<(),Response> + Send + Sync + 'static>
        IntoBackFang<fn(&Response)->Result<(),Response>> for F {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, _| {
                    self(res)
                }))
            }
        }
        impl<F: Fn(&mut Response)->Result<(),Response> + Send + Sync + 'static>
        IntoBackFang<fn(&mut Response)->Result<(),Response>> for F {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, _| {
                    self(res)
                }))
            }
        }

        impl<F: Fn(&Response, &Request)->Result<(),Response> + Send + Sync + 'static>
        IntoBackFang<fn(&Response, &Request)->Result<(),Response>> for F {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(move |res, req| {
                    self(res, req)
                }))
            }
        }
        impl<F: Fn(&mut Response, &Request)->Result<(),Response> + Send + Sync + 'static>
        IntoBackFang<fn(&mut Response, &Request)->Result<(),Response>> for F {
            fn into_back(self) -> proc::BackFang {
                proc::BackFang(Arc::new(
                    self
                ))
            }
        }
    };
};
