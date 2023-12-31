#![allow(non_snake_case)]

use std::{sync::Arc, any::Any};
use crate::{
    Context,
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
    impl<F: Fn(&mut Context) + Send + Sync + 'static>
    IntoFang<fn(&mut Context)> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, _| {
                        self(c);
                        Ok(())
                    }
                ))),
            })
        }
    }
    impl<F: Fn(&Context) + Send + Sync + 'static>
    IntoFang<fn(&Context)> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, _| {
                        self(c);
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
                    move |_, req| {
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
                    move |_, req| {
                        self(req);
                        Ok(())
                    }
                ))),
            })
        }
    }

    impl<F: Fn(&mut Context, &mut Request) + Send + Sync + 'static>
    IntoFang<fn(&mut Context, &mut Request)> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(c, req);
                        Ok(())
                    }
                ))),
            })
        }
    }
    impl<F: Fn(&mut Context, &Request) + Send + Sync + 'static>
    IntoFang<fn(&mut Context, &Request)> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(c, req);
                        Ok(())
                    }
                ))),
            })
        }
    }
    impl<F: Fn(&Context, &mut Request) + Send + Sync + 'static>
    IntoFang<fn(&Context, &mut Request)> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(c, req);
                        Ok(())
                    }
                ))),
            })
        }
    }
    impl<F: Fn(&Context, &Request) + Send + Sync + 'static>
    IntoFang<fn(&Context, &Request)> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(c, req);
                        Ok(())
                    }
                ))),
            })
        }
    }
};

const _: (/* Front; returning Result */) = {
    impl<F: Fn(&mut Context) -> Result<(), Response> + Send + Sync + 'static>
    IntoFang<fn(&mut Context) -> Result<(), Response>> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, _| {
                        self(c)
                    }
                ))),
            })
        }
    }
    impl<F: Fn(&Context) -> Result<(), Response> + Send + Sync + 'static>
    IntoFang<fn(&Context) -> Result<(), Response>> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, _| {
                        self(c)
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
                    move |_, req| {
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
                    move |_, req| {
                        self(req)
                    }
                ))),
            })
        }
    }

    impl<F: Fn(&mut Context, &mut Request) -> Result<(), Response> + Send + Sync + 'static>
    IntoFang<fn(&mut Context, &mut Request) -> Result<(), Response>> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(c, req)
                    }
                ))),
            })
        }
    }
    impl<F: Fn(&mut Context, &Request) -> Result<(), Response> + Send + Sync + 'static>
    IntoFang<fn(&mut Context, &Request) -> Result<(), Response>> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(c, req)
                    }
                ))),
            })
        }
    }
    impl<F: Fn(&Context, &mut Request) -> Result<(), Response> + Send + Sync + 'static>
    IntoFang<fn(&Context, &mut Request) -> Result<(), Response>> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(c, req)
                    }
                ))),
            })
        }
    }
    impl<F: Fn(&Context, &Request) -> Result<(), Response> + Send + Sync + 'static>
    IntoFang<fn(&Context, &Request) -> Result<(), Response>> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(c, req)
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
