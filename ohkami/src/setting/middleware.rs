use std::{pin::Pin, future::Future};
use crate::{context::Context, testing::Method, utils::validation, prelude::Response};


// const MIDDLEWARE_STORE_SIZE: usize = 16; // methos ごとに分かれてるのでまあ足りるかなと

pub(crate) type BeforeMiddleware = Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Context> + Send>> + Send + Sync>;
pub(crate) struct BeforeMiddlewareStore(Vec::<BeforeMiddleware>);
impl BeforeMiddlewareStore {
    fn store<
        F:   Clone + Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Context> + Send + 'static,
    >(f: F) -> Self {
        let (
            f1,  f2,  f3,  f4,
            f5,  f6,  f7,  f8,
            f9,  f10, f11, f12,
            f13, f14, f15, f16,
        ) = (
            f.clone(), f.clone(), f.clone(), f.clone(),
            f.clone(), f.clone(), f.clone(), f.clone(),
            f.clone(), f.clone(), f.clone(), f.clone(),
            f.clone(), f.clone(), f.clone(), f.clone(),
        );
        Self(vec![
            Box::new(move |ctx| Box::pin(f1(ctx))),  Box::new(move |ctx| Box::pin(f2(ctx))),
            Box::new(move |ctx| Box::pin(f3(ctx))),  Box::new(move |ctx| Box::pin(f4(ctx))),
            Box::new(move |ctx| Box::pin(f5(ctx))),  Box::new(move |ctx| Box::pin(f6(ctx))),
            Box::new(move |ctx| Box::pin(f7(ctx))),  Box::new(move |ctx| Box::pin(f8(ctx))),
            Box::new(move |ctx| Box::pin(f9(ctx))),  Box::new(move |ctx| Box::pin(f10(ctx))),
            Box::new(move |ctx| Box::pin(f11(ctx))), Box::new(move |ctx| Box::pin(f12(ctx))),
            Box::new(move |ctx| Box::pin(f13(ctx))), Box::new(move |ctx| Box::pin(f14(ctx))),
            Box::new(move |ctx| Box::pin(f15(ctx))), Box::new(move |ctx| Box::pin(f16(ctx))),
        ])
    }
    pub(crate) fn pop(&mut self) -> Option<BeforeMiddleware> {
        self.0.pop()
    }
}

pub(crate) type AfterMiddleware = Box<dyn Fn(Response) -> Pin<Box<dyn Future<Output=Response> + Send>> + Send + Sync>;
pub(crate) struct AfterMiddlewareStore(Vec::<AfterMiddleware>); impl AfterMiddlewareStore {
    fn store<
        F:   Clone + Fn(Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    >(f: F) -> Self {
        let (
            f1,  f2,  f3,  f4,
            f5,  f6,  f7,  f8,
            f9,  f10, f11, f12,
            f13, f14, f15, f16,
        ) = (
            f.clone(), f.clone(), f.clone(), f.clone(),
            f.clone(), f.clone(), f.clone(), f.clone(),
            f.clone(), f.clone(), f.clone(), f.clone(),
            f.clone(), f.clone(), f.clone(), f.clone(),
        );
        Self(vec![
            Box::new(move |res| Box::pin(f1(res))),  Box::new(move |res| Box::pin(f2(res))),
            Box::new(move |res| Box::pin(f3(res))),  Box::new(move |res| Box::pin(f4(res))),
            Box::new(move |res| Box::pin(f5(res))),  Box::new(move |res| Box::pin(f6(res))),
            Box::new(move |res| Box::pin(f7(res))),  Box::new(move |res| Box::pin(f8(res))),
            Box::new(move |res| Box::pin(f9(res))),  Box::new(move |res| Box::pin(f10(res))),
            Box::new(move |res| Box::pin(f11(res))), Box::new(move |res| Box::pin(f12(res))),
            Box::new(move |res| Box::pin(f13(res))), Box::new(move |res| Box::pin(f14(res))),
            Box::new(move |res| Box::pin(f15(res))), Box::new(move |res| Box::pin(f16(res))),
        ])
    }
    pub(crate) fn pop(&mut self) -> Option<AfterMiddleware> {
        self.0.pop()
    }
}

/// A set of ohkami's middlewares
pub struct Middleware {
    pub(crate) before:       Vec<(Method, /*route*/&'static str, BeforeMiddlewareStore, /*is from ANY*/bool)>,
    pub(crate) after:        Vec<(Method, /*route*/&'static str, AfterMiddlewareStore, /*is from ANY*/bool)>,
    pub(crate) setup_errors: Vec<String>,
} impl Middleware {
    pub fn new() -> Self {
        Self {
            before:       Vec::new(),
            after:        Vec::new(),
            setup_errors: Vec::new(),
        }
    }

    /// Add a middleware func for requests of any methods to given path
    #[allow(non_snake_case)]
    pub fn beforeANY<
        F:   Clone + Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Context> + Send + 'static,
    >(
        mut    self,
        route: &'static str,
        f:     F,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.before.push((Method::GET,    route, BeforeMiddlewareStore::store(f.clone()), true));
        self.before.push((Method::POST,   route, BeforeMiddlewareStore::store(f.clone()), true));
        self.before.push((Method::PATCH,  route, BeforeMiddlewareStore::store(f.clone()), true));
        self.before.push((Method::DELETE, route, BeforeMiddlewareStore::store(f.clone()), true));
        self
    }
    #[allow(non_snake_case)]
    pub fn afterANY<
        F:   Clone + Fn(Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    >(
        mut    self,
        route: &'static str,
        f:     F,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.after.push((Method::GET,    route, AfterMiddlewareStore::store(f.clone()), true));
        self.after.push((Method::POST,   route, AfterMiddlewareStore::store(f.clone()), true));
        self.after.push((Method::PATCH,  route, AfterMiddlewareStore::store(f.clone()), true));
        self.after.push((Method::DELETE, route, AfterMiddlewareStore::store(f.clone()), true));
        self
    }

    /// Add a middleware func for requests of GET requests to given path
    #[allow(non_snake_case)]
    pub fn beforeGET<
        F:   Clone + Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Context> + Send + 'static,
    >(
        mut self,
        route: &'static str,
        f:     F,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.before.push((Method::GET, route, BeforeMiddlewareStore::store(f), false));
        self
    }
    #[allow(non_snake_case)]
    pub fn afterGET<
        F:   Clone + Fn(Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    >(
        mut self,
        route: &'static str,
        f:     F,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.after.push((Method::GET, route, AfterMiddlewareStore::store(f), false));
        self
    }

    /// Add a middleware func for requests of POST requests to given path
    #[allow(non_snake_case)]
    pub fn beforePOST<
        F:   Clone + Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Context> + Send + 'static,
    >(
        mut self,
        route: &'static str,
        f:     F,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.before.push((Method::POST, route, BeforeMiddlewareStore::store(f), false));
        self
    }
    #[allow(non_snake_case)]
    pub fn afterPOST<
        F:   Clone + Fn(Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    >(
        mut self,
        route: &'static str,
        f:     F,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.after.push((Method::POST, route, AfterMiddlewareStore::store(f), false));
        self
    }

    /// Add a middleware func for requests of PATCH requests to given path
    #[allow(non_snake_case)]
    pub fn beforePATCH<
        F:   Clone + Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Context> + Send + 'static,
    >(
        mut self,
        route: &'static str,
        f:     F,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.before.push((Method::PATCH, route, BeforeMiddlewareStore::store(f), false));
        self
    }
    #[allow(non_snake_case)]
    pub fn afterPATCH<
        F:   Clone + Fn(Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    >(
        mut self,
        route: &'static str,
        f:     F,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.after.push((Method::PATCH, route, AfterMiddlewareStore::store(f), false));
        self
    }

    /// Add a middleware func for requests of DELETE requests to given path
    #[allow(non_snake_case)]
    pub fn beforeDELETE<
        F:   Clone + Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Context> + Send + 'static,
    >(
        mut self,
        route: &'static str,
        f:     F,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.before.push((Method::DELETE, route, BeforeMiddlewareStore::store(f), false));
        self
    }
    #[allow(non_snake_case)]
    pub fn afterDELETE<
        F:   Clone + Fn(Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    >(
        mut self,
        route: &'static str,
        f:     F,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.after.push((Method::DELETE, route, AfterMiddlewareStore::store(f), false));
        self
    }

    pub(crate) fn merge(mut self, mut another: Self) -> Self {
        self.before.append(&mut another.before);
        self.after.append(&mut another.after);
        self.setup_errors.append(&mut another.setup_errors);
        self
    }
}


#[cfg(test)]
mod test {
    use crate::prelude::*;

    async fn cors(mut c: Context) -> Context {
        c.add_header(Header::AccessControlAllowOrigin, "localhost:8000");
        c
    }

    #[test]
    fn server() {
        let middleware = Middleware::new()
            .beforeANY("/api/*", cors);

        Ohkami::with(middleware)
            .GET("/api", hello)
            .GET("/api/sleepy", sleepy_hello);
    }

    async fn hello(c: Context) -> Result<Response> {
        c.OK("Hello!")
    }

    async fn sleepy_hello(c: Context, time: u64) -> Result<Response> {
        (time < 10)
            ._else(|| c.BadRequest("`time` must be less than 10"))?;
        std::thread::sleep(
            std::time::Duration::from_secs(time)
        );
        c.OK("Hello, I'm sleepy...")
    }
}
