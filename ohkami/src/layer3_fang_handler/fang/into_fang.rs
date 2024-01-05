#![allow(non_snake_case)]

use std::{sync::Arc, any::Any};
use crate::{
    Request,
    Response,
    layer3_fang_handler::{FrontFang, FangProc, BackFang, Fang},
};


pub trait IntoFang<Args> {
    /// Why Option: returns `None` when executing
    /// some builtin fangs like `cors`. In other words,
    /// this **internally executes** proc that returns `None`.
    fn into_fang(self) -> Option<Fang>;
}

const _: (/* Front; not retuning Result */) = {
    impl<F: Fn() + Send + Sync + 'static>
    IntoFang<fn()> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |_| {
                        self();
                        Ok(())
                    }
                ))),
            })
        }
    }

    impl<F: Fn(&mut Request) + Send + Sync + 'static>
    IntoFang<fn(&mut Request)> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |req| {
                        self(req);
                        Ok(())
                    }
                ))),
            })
        }
    }

    impl<F: Fn(&Request) + Send + Sync + 'static>
    IntoFang<fn(&Request)> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |req| {
                        self(req);
                        Ok(())
                    }
                ))),
            })
        }
    }
};

const _: (/* Front; returning Result */) = {
    impl<F: Fn() -> Result<(), Response> + Send + Sync + 'static>
    IntoFang<fn() -> Result<(), Response>> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |_| {
                        self()
                    }
                ))),
            })
        }
    }

    impl<F: Fn(&mut Request) -> Result<(), Response> + Send + Sync + 'static>
    IntoFang<fn(&mut Request) -> Result<(), Response>> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |req| {
                        self(req)
                    }
                ))),
            })
        }
    }

    impl<F: Fn(&Request) -> Result<(), Response> + Send + Sync + 'static>
    IntoFang<fn(&Request) -> Result<(), Response>> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |req| {
                        self(req)
                    }
                ))),
            })
        }
    }
};

const _: (/* Back */) = {
    impl<F: Fn(Response)->Response + Send + Sync + 'static>
    IntoFang<fn(Response)->Response> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Back(BackFang(Arc::new(
                    move |res| {
                        self(res)
                    }
                ))),
            })
        }
    }

    impl<F: Fn(&Response) + Send + Sync + 'static>
    IntoFang<fn(&Response)> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Back(BackFang(Arc::new(
                    move |res| {
                        self(&res);
                        res
                    }
                ))),
            })
        }
    }
};
