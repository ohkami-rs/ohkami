use std::{pin::Pin, future::Future};
use crate::{context::Context, testing::Method, utils::validation};


pub(crate) type MiddlewareFunc = Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Context> + Send>> + Send + Sync>;

pub trait MiddlewareArg {}
pub trait MiddlewareProcess<Arg: MiddlewareArg> {
    fn into_middleware_func(self) -> MiddlewareFunc;
}

impl MiddlewareArg for Context {}
impl<F, Fut> MiddlewareProcess<Context> for F
where
    F:   Fn(Context) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Context> + Send + 'static,
{
    fn into_middleware_func(self) -> MiddlewareFunc {
        Box::new(move |ctx| Box::pin(self(ctx)))
    }
}

/// A set of ohkami's middlewares
pub struct Middleware {
    pub(crate) proccess:     Vec<(Method, /*route*/&'static str, MiddlewareFunc)>,
    pub(crate) setup_errors: Vec<String>,
} impl Middleware {
    pub fn new() -> Self {
        Self {
            proccess: Vec::new(),
            setup_errors: Vec::new(),
        }
    }

    /// Add a middleware func for requests of any methods to given path
    #[allow(non_snake_case)]
    pub fn ANY<P: MiddlewareProcess<Arg> + Clone, Arg: MiddlewareArg>(
        mut self,
        route: &'static str,
        proccess: P,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.proccess.push((Method::GET, route, proccess.clone().into_middleware_func()));
        self.proccess.push((Method::POST, route, proccess.clone().into_middleware_func()));
        self.proccess.push((Method::PATCH, route, proccess.clone().into_middleware_func()));
        self.proccess.push((Method::DELETE, route, proccess.clone().into_middleware_func()));
        self
    }

    /// Add a middleware func for requests of GET requests to given path
    #[allow(non_snake_case)]
    pub fn GET<P: MiddlewareProcess<Arg>, Arg: MiddlewareArg>(
        mut self,
        route: &'static str,
        proccess: P,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.proccess.push((Method::GET, route, proccess.into_middleware_func()));
        self
    }

    /// Add a middleware func for requests of POST requests to given path
    #[allow(non_snake_case)]
    pub fn POST<P: MiddlewareProcess<Arg>, Arg: MiddlewareArg>(
        mut self,
        route: &'static str,
        proccess: P,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.proccess.push((Method::POST, route, proccess.into_middleware_func()));
        self
    }

    /// Add a middleware func for requests of PATCH requests to given path
    #[allow(non_snake_case)]
    pub fn PATCH<P: MiddlewareProcess<Arg>, Arg: MiddlewareArg>(
        mut self,
        route: &'static str,
        proccess: P,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.proccess.push((Method::PATCH, route, proccess.into_middleware_func()));
        self
    }

    /// Add a middleware func for requests of DELETE requests to given path
    #[allow(non_snake_case)]
    pub fn DELETE<P: MiddlewareProcess<Arg>, Arg: MiddlewareArg>(
        mut self,
        route: &'static str,
        proccess: P,
    ) -> Self {
        if ! validation::valid_middleware_route(route) {
            self.setup_errors.push(
                format!("middleware route `{route}` is invalid")
            )
        }
        self.proccess.push((Method::DELETE, route, proccess.into_middleware_func()));
        self
    }

    pub(crate) fn merge(mut self, mut another: Self) -> Self {
        self.proccess.append(&mut another.proccess);
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
            .ANY("/api/*", cors);

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
