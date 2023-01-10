use std::{pin::Pin, future::Future};
use crate::{context::Context, testing::Method, utils::validation, prelude::Response};


// pub(crate) type MiddlewareFunc = Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Context> + Send>> + Send + Sync>;
// 
// pub trait MiddlewareArg {}
// pub trait MiddlewareProcess<Arg: MiddlewareArg> {
//     fn into_middleware_func(self) -> MiddlewareFunc;
// }
// 
// impl MiddlewareArg for Context {}
// impl<F, Fut> MiddlewareProcess<Context> for F
// where
//     F:   Fn(Context) -> Fut + Send + Sync + 'static,
//     Fut: Future<Output = Context> + Send + 'static,
// {
//     fn into_middleware_func(self) -> MiddlewareFunc {
//         Box::new(move |ctx| Box::pin(self(ctx)))
//     }
// }
// 

const MIDDLEWARE_STORE_SIZE: usize = 16; // methos ごとに分かれてるのでまあ足りるかなと

pub(crate) struct BeforeMiddlewareStore(Vec::<
    Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Context> + Send>> + Send + Sync>
>); impl BeforeMiddlewareStore {
    fn store<
        F:   Clone + Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Context> + Send + 'static,
    >(f: F) -> Self {
        Self(vec![
            Box::new(move|ctx|Box::pin((f.clone())(ctx))),Box::new(move|ctx|Box::pin((f.clone())(ctx))),
            Box::new(move|ctx|Box::pin((f.clone())(ctx))),Box::new(move|ctx|Box::pin((f.clone())(ctx))),
            Box::new(move|ctx|Box::pin((f.clone())(ctx))),Box::new(move|ctx|Box::pin((f.clone())(ctx))),
            Box::new(move|ctx|Box::pin((f.clone())(ctx))),Box::new(move|ctx|Box::pin((f.clone())(ctx))),
            Box::new(move|ctx|Box::pin((f.clone())(ctx))),Box::new(move|ctx|Box::pin((f.clone())(ctx))),
            Box::new(move|ctx|Box::pin((f.clone())(ctx))),Box::new(move|ctx|Box::pin((f.clone())(ctx))),
            Box::new(move|ctx|Box::pin((f.clone())(ctx))),Box::new(move|ctx|Box::pin((f.clone())(ctx))),
            Box::new(move|ctx|Box::pin((f.clone())(ctx))),Box::new(move|ctx|Box::pin((f.clone())(ctx))),
        ])
    }
}

pub(crate) struct AfterMiddlewareStore(Vec::<
    Box<dyn Fn(Response) -> Pin<Box<dyn Future<Output=Response> + Send>> + Send + Sync>
>); impl AfterMiddlewareStore {
    fn store<
        F:   Clone + Fn(Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    >(f: F) -> Self {
        Self(vec![
            Box::new(move|res|Box::pin((f.clone())(res))),Box::new(move|res|Box::pin((f.clone())(res))),
            Box::new(move|res|Box::pin((f.clone())(res))),Box::new(move|res|Box::pin((f.clone())(res))),
            Box::new(move|res|Box::pin((f.clone())(res))),Box::new(move|res|Box::pin((f.clone())(res))),
            Box::new(move|res|Box::pin((f.clone())(res))),Box::new(move|res|Box::pin((f.clone())(res))),
            Box::new(move|res|Box::pin((f.clone())(res))),Box::new(move|res|Box::pin((f.clone())(res))),
            Box::new(move|res|Box::pin((f.clone())(res))),Box::new(move|res|Box::pin((f.clone())(res))),
            Box::new(move|res|Box::pin((f.clone())(res))),Box::new(move|res|Box::pin((f.clone())(res))),
            Box::new(move|res|Box::pin((f.clone())(res))),Box::new(move|res|Box::pin((f.clone())(res))),
        ])
    }
}

/// A set of ohkami's middlewares
pub struct Middleware {
    pub(crate) before:       Vec<(Method, /*route*/&'static str, BeforeMiddlewareStore)>,
    pub(crate) after:        Vec<(Method, /*route*/&'static str, AfterMiddlewareStore)>,
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
        self.before.push((Method::GET, route, BeforeMiddlewareStore::store(f.clone())));
        self.before.push((Method::POST, route, BeforeMiddlewareStore::store(f.clone())));
        self.before.push((Method::PATCH, route, BeforeMiddlewareStore::store(f.clone())));
        self.before.push((Method::DELETE, route, BeforeMiddlewareStore::store(f.clone())));
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
        self.after.push((Method::GET, route, AfterMiddlewareStore::store(f.clone())));
        self.after.push((Method::POST, route, AfterMiddlewareStore::store(f.clone())));
        self.after.push((Method::PATCH, route, AfterMiddlewareStore::store(f.clone())));
        self.after.push((Method::DELETE, route, AfterMiddlewareStore::store(f.clone())));
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
        self.before.push((Method::GET, route, BeforeMiddlewareStore::store(f)));
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
        self.after.push((Method::GET, route, AfterMiddlewareStore::store(f)));
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
        self.before.push((Method::POST, route, BeforeMiddlewareStore::store(f)));
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
        self.after.push((Method::POST, route, AfterMiddlewareStore::store(f)));
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
        self.before.push((Method::PATCH, route, BeforeMiddlewareStore::store(f)));
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
        self.after.push((Method::PATCH, route, AfterMiddlewareStore::store(f)));
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
        self.before.push((Method::DELETE, route, BeforeMiddlewareStore::store(f)));
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
        self.after.push((Method::DELETE, route, AfterMiddlewareStore::store(f)));
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
