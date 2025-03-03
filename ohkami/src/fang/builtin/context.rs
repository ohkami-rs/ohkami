use crate::{Request, Response, FromRequest};
use crate::fang::{FangAction, SendSyncOnNative};

/// # Request Context
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
#[derive(Clone)]
pub struct Context<'req, T: SendSyncOnNative + 'static>(pub &'req T);

impl<T: SendSyncOnNative + 'static> Context<'static, T>
where
    T: Clone
{
    pub fn new(data: T) -> impl FangAction {
        return ContextAction(data);

        #[derive(Clone)]
        struct ContextAction<T: Clone + SendSyncOnNative + 'static>(T);

        impl<T: Clone + SendSyncOnNative + 'static> FangAction for ContextAction<T> {
            #[inline]
            async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
                req.context.set(self.0.clone());
                Ok(())
            }
        }
    }
}

impl<'req, T: SendSyncOnNative + 'static> FromRequest<'req> for Context<'req, T> {
    type Error = std::convert::Infallible;

    #[inline]
    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        match req.context.get::<T>() {
            Some(d) => Some(Ok(Self(d))),
            None => {
                #[cfg(debug_assertions)] {
                    crate::WARNING!(
                        "Context of `{}` doesn't exist",
                        std::any::type_name::<T>()
                    )
                }
                None
            }
        }
    }
}


#[cfg(test)]
mod test {
    #[test]
    fn context_fang_bount() {
        use crate::fang::{Fang, BoxedFPC};
        fn assert_fang<T: Fang<BoxedFPC>>(_: T) {}

        assert_fang(super::Context::new(String::new()));
    }
}
