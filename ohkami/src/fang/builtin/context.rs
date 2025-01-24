/// # Request Contexts
/// 
/// Memorize and retrieve any data within a request.
/// 
/// <br>
/// 
/// ```no_run
/// use ohkami::prelude::*;
/// use std::sync::Arc;
/// 
/// #[tokio::main]
/// async fn main() {
///     let sample_data = Arc::new(String::from("ohkami"));
/// 
///     Ohkami::new((
///         Context::new(sample_data), // <--
///         "/hello"
///             .GET(hello),
///     )).howl("0.0.0.0:8080").await
/// }
/// 
/// async fn hello(
///     Context(name): Context<'_, Arc<String>>, // <--
/// ) -> String {
///     format!("Hello, {name}!")
/// }
/// ```
pub struct Context<'req, T: Send + Sync + 'static>(pub &'req T);

impl<'req, T: Send + Sync + 'static> crate::FromRequest<'req> for Context<'req, T> {
    type Error = std::convert::Infallible;

    #[inline]
    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        match req.memorized::<T>() {
            Some(d) => Some(Ok(Self(d))),
            None => {
                #[cfg(debug_assertions)] {
                    crate::warning!(
                        "Context of `{}` doesn't exist",
                        std::any::type_name::<T>()
                    )
                }
                None
            }
        }
    }
}

const _: () = {
    use crate::fang::{Fang, FangProc};

    impl<'req, Data: Clone + Send + Sync + 'static> Context<'req, Data> {
        #[allow(private_interfaces)]
        pub fn new(data: Data) -> UseContext<Data> {
            UseContext(data)
        }
    }

    pub struct UseContext<Data: Clone + Send + Sync + 'static>(
        Data
    );
    impl<Data: Clone + Send + Sync + 'static, Inner: FangProc>
    Fang<Inner> for UseContext<Data> {
        type Proc = UseContextProc<Data, Inner>;
        fn chain(&self, inner: Inner) -> Self::Proc {
            UseContextProc { data: self.0.clone(), inner }
        }
    }

    pub struct UseContextProc<
        Data:  Clone + Send + Sync + 'static,
        Inner: FangProc,
    > {
        data:  Data,
        inner: Inner,
    }
    impl<Data: Clone + Send + Sync + 'static, Inner: FangProc>
    FangProc for UseContextProc<Data, Inner> {
        #[cfg(not(feature="rt_worker"))]
        #[inline]
        fn bite<'b>(&'b self, req: &'b mut crate::Request) -> impl std::future::Future<Output = crate::Response> + Send {
            req.memorize(self.data.clone());
            self.inner.bite(req)
        }
        #[cfg(feature="rt_worker")]
        #[inline]
        fn bite<'b>(&'b self, req: &'b mut crate::Request) -> impl std::future::Future<Output = crate::Response> {
            req.memorize(self.data.clone());
            self.inner.bite(req)
        }
    }
};
