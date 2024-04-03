use super::super::{Fang, FangProc};
use crate::{Request, Response};


pub struct ForeFang(
    pub fn(&mut Request)
); const _: () = {
    impl<I: FangProc> Fang<I> for ForeFang {
        type Proc = ForeFangProc<I>;
        fn chain(&self, inner: I) -> Self::Proc {
            ForeFangProc { proc: self.0, inner }
        }
    }

    pub struct ForeFangProc<I: FangProc> {
        proc:  fn(&mut Request),
        inner: I
    }
    impl<I: FangProc> FangProc for ForeFangProc<I> {
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send {
            (self.proc)(req);
            self.inner.bite(req)
        }
    }
};

pub struct FrontFang(
    pub fn(&mut Request) -> Result<(), Response>
); const _: () = {
    impl<I: FangProc> Fang<I> for FrontFang {
        type Proc = FrontFangProc<I>;
        fn chain(&self, inner: I) -> Self::Proc {
            FrontFangProc { proc: self.0, inner }
        }
    }

    pub struct FrontFangProc<I: FangProc> {
        proc:  fn(&mut Request) -> Result<(), Response>,
        inner: I
    }
    impl<I: FangProc> FangProc for FrontFangProc<I> {
        async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
            match (self.proc)(req) {
                Ok(()) => self.inner.bite(req).await,
                Err(e) => e,
            }
        }
    }
};

pub struct BackFang(
    pub fn(&mut Response)
); const _: () = {
    impl<I: FangProc> Fang<I> for BackFang {
        type Proc = BackFangProc<I>;
        fn chain(&self, inner: I) -> Self::Proc {
            BackFangProc { proc: self.0, inner }
        }
    }

    pub struct BackFangProc<I: FangProc> {
        proc:  fn(&mut Response),
        inner: I,
    }
    impl<I: FangProc> FangProc for BackFangProc<I> {
        async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
            let mut res = self.inner.bite(req).await;
            (self.proc)(&mut res);
            res
        }
    }
};


/*
pub struct BackFang<
    Proc: for<'f> BackFangFn<'f>,
>(pub Proc);

trait BackFangFn<'b>: Clone + Send + Sync + 'static {
    fn bite_fn(&'b self, res: &'b mut Response) -> impl Future<Output = ()> + Send + 'b;
}
impl<'f,
    F:   Fn(&'f mut Response) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'f,
> BackFangFn<'f> for F {
    fn bite_fn(&'f self, res: &'f mut Response) -> impl Future<Output = ()> + Send + 'f {
        self(res)
    }
}

const _: () = {
    impl<
        Proc: for<'f> BackFangFn<'f>,
        I:    FangProc,
    > Fang<I> for BackFang<Proc> {
        type Proc = BackFangProc<Proc, I>;
        fn chain(&self, inner: I) -> Self::Proc {
            BackFangProc {
                proc: self.0.clone(),
                inner
            }
        }
    }

    pub struct BackFangProc<
        Proc: for<'f> BackFangFn<'f>,
        I:    FangProc,
    > {
        proc:  Proc,
        inner: I,
    }


    impl<
        Proc: for<'f> BackFangFn<'f>,
        I:    FangProc,
    > FangProc for BackFangProc<Proc, I> {
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl Future<Output = Response> + Send + 'b {
            struct BackFangProcFuture<'b,
                // Proc: for<'f> BackFangFn<'f>,
                // I:    FangProc,
                InnerFuture: Future<Output = Response> + Send + 'b,
                ProcFuture:  Future<Output = Response> + Send + 'b,
            > {
                inner_future: InnerFuture,
                proc_future:  Option<ProcFuture>,
                __lifetime__: PhantomData<&'b ()>
            }

            impl<'b,
                InnerFuture: Future<Output = Response> + Send + 'b,
                ProcFuture:  Future<Output = Response> + Send + 'b,
            > Future for BackFangProcFuture<'b, InnerFuture, ProcFuture> {
                type Output = Response;
                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    /*
                        let mut res = self.inner.bite(req).await;
                        self.proc.bite_fn(&mut res).await;
                        res
                    */
                
                    todo!()
                }
            }

            BackFangProcFuture {
                inner_future: self.inner.bite(req),
                proc_future:  None,
                __lifetime__: PhantomData
            }
        }
    }
};
*/


#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;
    use crate::testing::*;

    #[crate::__rt__::test]
    async fn availablity() {
        use std::sync::{Mutex, OnceLock};

        fn messages() -> &'static Mutex<Vec<String>> {
            static MESSAGES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
            MESSAGES.get_or_init(|| Mutex::new(Vec::new()))
        }

        fn hi_fang(_: &mut Request) -> Result<(), Response> {
            messages().lock().unwrap().push(format!("Hi!"));
            Ok(())
        }
        fn bye_fang(_: &mut Response) {
            messages().lock().unwrap().push(format!("Bye!"));
        }

        #[derive(Clone)]
        struct GreetingFang { name: &'static str }
        const _: () = {
            impl<I: FangProc> Fang<I> for GreetingFang {
                type Proc = GreetingFangProc<I>;
                fn chain(&self, inner: I) -> Self::Proc {
                    GreetingFangProc { fang: self.clone(), inner }
                }
            }

            struct GreetingFangProc<I: FangProc> {
                fang:  GreetingFang,
                inner: I
            }
            impl<I: FangProc> FangProc for GreetingFangProc<I> {
                async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
                    {
                        messages().lock().unwrap().push(format!("Hello, {}!", self.fang.name));
                    }
                    let res = self.inner.bite(req).await;
                    {
                        messages().lock().unwrap().push(format!("Bye, {}!", self.fang.name));
                    }
                    res
                }
            }
        };

        let t = Ohkami::with((
            FrontFang(hi_fang),
            BackFang(bye_fang),
            FrontFang(|_| {
                messages().lock().unwrap().push(format!("Hello, Alice!"));
                Ok(())
            }),
            GreetingFang { name: "Clerk" },
        ), (
            "/greet".POST(|| async {"Hi, I'm Handler!"}),
        ));

        {
            let req = TestRequest::POST("/greet");
            let res = t.oneshot(req).await;

            assert_eq!(res.status(), Status::OK);
            assert_eq!(&*messages().lock().unwrap(), &[
                "Hi!",
                "Hello, Alice!",
                "Hello, Clerk!",
                "Bye, Clerk!",
                "Bye!",
            ]);
        }
    }
}
