#![allow(non_snake_case)]

use std::{future::Future, sync::Arc, any::Any};
use super::{Fang};
use crate::{
    Context,
    Request,
    Response,
    layer3_fang_handler::{FrontFang, FangProc, BackFang},
};


pub trait IntoFang<Args> {
    fn into_fang(self) -> Fang;
}

const _: (/* Front: not retuning Result */) = {
    impl IntoFang<(&Context,)> for fn(&Context) {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(&c);
                        Ok((c, req))
                    }
                ))),
            }
        }
    }

    impl IntoFang<(&mut Context,)> for fn(&mut Context) {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |mut c, req| {
                        self(&mut c);
                        Ok((c, req))
                    }
                ))),
            }
        }
    }

    impl IntoFang<(&Request,)> for fn(&Request) {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(&req);
                        Ok((c, req))
                    }
                ))),
            }
        }
    }

    impl IntoFang<(&Context, &Request)> for fn(&Context, &Request) {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(&c, &req);
                        Ok((c, req))
                    }
                ))),
            }
        }
    }

    impl IntoFang<(&mut Context, &Request)> for fn(&mut Context, &Request) {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |mut c, req| {
                        self(&mut c, &req);
                        Ok((c, req))
                    }
                ))),
            }
        }
    }
};

const _: (/* Front: returning Result */) = {
    impl IntoFang<(&Context, ())> for fn(&Context) -> Result<(), Response> {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(&c)?;
                        Ok((c, req))
                    }
                ))),
            }
        }
    }

    impl IntoFang<(&mut Context, ())> for fn(&mut Context) -> Result<(), Response> {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |mut c, req| {
                        self(&mut c)?;
                        Ok((c, req))
                    }
                ))),
            }
        }
    }

    impl IntoFang<(&Request, ())> for fn(&Request) -> Result<(), Response> {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(&req)?;
                        Ok((c, req))
                    }
                ))),
            }
        }
    }

    impl IntoFang<(&Context, &Request, ())> for fn(&Context, &Request) -> Result<(), Response> {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |c, req| {
                        self(&c, &req)?;
                        Ok((c, req))
                    }
                ))),
            }
        }
    }

    impl IntoFang<(&mut Context, &Request, ())> for fn(&mut Context, &Request) -> Result<(), Response> {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |mut c, req| {
                        self(&mut c, &req)?;
                        Ok((c, req))
                    }
                ))),
            }
        }
    }
};

const _: (/* Back */) = {
    impl IntoFang<(&Response,)> for fn(&Response) {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Back(BackFang(Arc::new(
                    move |res| {
                        self(&res);
                        res
                    }
                ))),
            }
        }
    }

    impl IntoFang<(&mut Response,)> for fn(&mut Response) {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Back(BackFang(Arc::new(
                    move |mut res| {
                        self(&mut res);
                        res
                    }
                ))),
            }
        }
    }

    impl IntoFang<(&Response, ())> for fn(&Response) -> Result<(), Response> {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Back(BackFang(Arc::new(
                    move |res| {
                        match self(&res) {
                            Ok(_)  => res,
                            Err(e) => e,
                        }
                    }
                ))),
            }
        }
    }

    impl IntoFang<(&mut Response, ())> for fn(&mut Response) -> Result<(), Response> {
        fn into_fang(self) -> Fang {
            Fang {
                id:   self.type_id(),
                proc: FangProc::Back(BackFang(Arc::new(
                    move |mut res| {
                        match self(&mut res) {
                            Ok(_)  => res,
                            Err(e) => e,
                        }
                    }
                ))),
            }
        }
    }
};
