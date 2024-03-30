use super::{Fang, FangProc};
use crate::{Request, Response};
use std::future::Future;


pub struct FrontFang<
    Proc: Fn(&mut Request) -> Fut + Clone + Send + Sync + 'static,
    Fut:  Future<Output = Result<(), Response>> + Send + 'static,
>(Proc);

const _: () = {
    pub struct FrontFangProc<
        Proc: Fn(&mut Request) -> Fut + Clone + Send + Sync + 'static,
        Fut:  Future<Output = Result<(), Response>> + Send + 'static,
        I:    FangProc,
    > {
        proc:  FrontFang<Proc, Fut>,
        inner: I,
    }

    impl<
        Proc: Fn(&mut Request) -> Fut + Clone + Send + Sync + 'static,
        Fut:  Future<Output = Result<(), Response>> + Send + 'static,
        I:    FangProc,
    > FangProc for FrontFangProc<Proc, Fut, I> {
        async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
            match (self.proc.0)(req).await {
                Ok(()) => self.inner.bite(req).await,
                Err(e) => e,
            }
        }
    }

    impl<
        Proc: Fn(&mut Request) -> Fut + Clone + Send + Sync + 'static,
        Fut:  Future<Output = Result<(), Response>> + Send + 'static,
        I:    FangProc,
    > Fang<I> for FrontFang<Proc, Fut> {
        type Proc = FrontFangProc<Proc, Fut, I>;
        fn chain(&self, inner: I) -> Self::Proc {
            FrontFangProc {
                proc: FrontFang(self.0.clone()),
                inner
            }
        }
    }
};


pub struct BackFang<
    Proc: Fn(&mut Response) -> Fut + Clone + Send + Sync + 'static,
    Fut:  Future<Output = ()> + Send + 'static,
>(Proc);

const _: () = {
    pub struct BackFangProc<
        Proc: Fn(&mut Response) -> Fut + Clone + Send + Sync + 'static,
        Fut:  Future<Output = ()> + Send + 'static,
        I:    FangProc,
    > {
        proc:  BackFang<Proc, Fut>,
        inner: I,
    }

    impl<
        Proc: Fn(&mut Response) -> Fut + Clone + Send + Sync + 'static,
        Fut:  Future<Output = ()> + Send + 'static,
        I:    FangProc,
    > FangProc for BackFangProc<Proc, Fut, I> {
        async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
            let mut res = self.inner.bite(req).await;
            (self.proc.0)(&mut res).await;
            res
        }
    }

    impl<
        Proc: Fn(&mut Response) -> Fut + Clone + Send + Sync + 'static,
        Fut:  Future<Output = ()> + Send + 'static,
        I:    FangProc,
    > Fang<I> for BackFang<Proc, Fut> {
        type Proc = BackFangProc<Proc, Fut, I>;
        fn chain(&self, inner: I) -> Self::Proc {
            BackFangProc {
                proc: BackFang(self.0.clone()),
                inner
            }
        }
    }
};


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

        let t = Ohkami::with((
            FrontFang(|_| async {
                messages().lock().unwrap().push(format!("Hello, Alice!"));
                Ok(())
            }),
            BackFang(|_| async {
                messages().lock().unwrap().push(format!("Bye, Clerk!"));
            }),
            FrontFang(|_| async {
                messages().lock().unwrap().push(format!("Hello, Bob!"));
                Ok(())
            }),
        ), (
            "/greet".POST(|| async {"Hi, I'm Handler!"}),
        ));

        {
            let req = TestRequest::POST("/greet");
            let res = t.oneshot(req).await;

            assert_eq!(res.status(), Status::OK);
            assert_eq!(&*messages().lock().unwrap(), &[
                "Hello, Alice!",
                "Hello, Bob!",
                "Bye, Clerk!",
            ]);
        }
    }
}
