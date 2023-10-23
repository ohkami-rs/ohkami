use std::{pin::Pin, future::Future};
use crate::{Response, Request};

#[cfg(test)] use crate::{Ohkami, Context};

pub trait Testing {
    fn oneshot<'test>(&self, req: &'test mut Request) -> TestResponse<'test>;
}

pub struct TestResponse<'test>(
    Box<dyn Future<Output = Response> + 'test>
); impl<'test> Future for TestResponse<'test> {
    type Output = Response;
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        unsafe {self.map_unchecked_mut(|tr| tr.0.as_mut())}
            .poll(cx)
    }
}

#[cfg(test)]
impl Testing for Ohkami {
    fn oneshot<'test>(&self, req: &'test mut Request) -> TestResponse<'test> {
        let router = {
            let mut router = self.routes.clone();
            for (methods, fang) in &self.fangs {
                router = router.apply_fang(methods, fang.clone())
            }
            router.into_radix()
        };

        let res = async move {router.handle(Context::new(), req).await};
        TestResponse(Box::new(res))
    }
}
