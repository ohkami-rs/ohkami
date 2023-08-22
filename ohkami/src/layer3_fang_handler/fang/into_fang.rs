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

const _: (/* Front: not retuning Result */) = {
    impl<F: Fn(&mut Context, Request)->Request + Send + Sync + 'static>
    IntoFang<fn(&mut Context, Request)->Request> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |mut c, mut req| {
                        req = self(&mut c, req);
                        Ok((c, req))
                    }
                ))),
            })
        }
    }
};

const _: (/* Front: returning Result */) = {
    impl<F: Fn(&mut Context, Request) -> Result<Request, Response> + Send + Sync + 'static>
    IntoFang<fn(&mut Context, Request) -> Result<Request, Response>> for F {
        fn into_fang(self) -> Option<Fang> {
            Some(Fang {
                id:   self.type_id(),
                proc: FangProc::Front(FrontFang(Arc::new(
                    move |mut c, mut req| {
                        req = self(&mut c, req)?;
                        Ok((c, req))
                    }
                ))),
            })
        }
    }
};

const _: (/* Back */) = {
    impl<F: Fn(Response)->Response + Send + Sync + 'static>
    IntoFang<(&Response,)> for F {
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
};
