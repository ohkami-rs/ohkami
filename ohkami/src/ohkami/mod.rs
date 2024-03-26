#![cfg(any(feature="rt_tokio",feature="rt_async-std"))]

pub(crate) mod router;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use router::TrieRouter;

mod timeout;
mod build;
mod howl;

use crate::fang::{Fangs, FangProc};
use crate::{Method, Request, Response};


/// # Ohkami - a robust wolf who serves your web app
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// # use ohkami::prelude::*;
/// # use ohkami::serde::Serialize;
/// # use ohkami::typed::Payload;
/// # use ohkami::typed::status::{OK, Created};
/// # use ohkami::builtin::payload::JSON;
/// # 
/// 
/// struct Auth;
/// impl FrontFang for Auth {
///     /* 〜 */
/// #    type Error = Response;
/// #    async fn bite(&self, req: &mut Request) -> Result<(), Self::Error> {
/// #        // Do something...
/// #
/// #        Ok(())
/// #    }
/// }
/// 
/// # #[Payload(JSON/S)]
/// # struct User {
/// #     id:   usize,
/// #     name: String,
/// #     age:  Option<usize>,
/// # }
/// # 
/// # enum APIError {
/// #     UserNotFound
/// # }
/// # impl IntoResponse for APIError {
/// #     fn into_response(self) -> Response {
/// #         match self {
/// #             Self::UserNotFound => Response::InternalServerError()
/// #         }
/// #     }
/// # }
/// # 
/// # async fn health_check() -> impl IntoResponse {
/// #     Status::NoContent
/// # }
/// # 
/// # async fn create_user() -> Created<User> {
/// #     Created(User {
/// #         id:   42,
/// #         name: String::from("ohkami"),
/// #         age:  None,
/// #     })
/// # }
/// # 
/// # async fn get_user_by_id(id: usize) -> Result<OK<User>, APIError> {
/// #     Ok(OK(User {
/// #         id,
/// #         name: String::from("ohkami"),
/// #         age:  Some(2),
/// #     }))
/// # }
/// # 
/// # async fn update_user(id: usize) -> impl IntoResponse {
/// #     Status::OK
/// # }
/// 
/// fn my_ohkami() -> Ohkami {
///     let api_ohkami = Ohkami::with((Auth,), (
///         "/users"
///             .POST(create_user),
///         "/users/:id"
///             .GET(get_user_by_id)
///             .PATCH(update_user),
///     ));
/// 
///     Ohkami::new((
///         "/hc" .GET(health_check),
///         "/api".By(api_ohkami),
///     ))
/// }
/// ```
/// 
/// <br>
/// 
/// #### handler schema：
/// `async ({path_params}?, {FromRequest type}s...) -> {IntoResponse type}`
/// 
/// #### path_params：
/// A tuple of types that implement `FromParam` trait.\
/// If the path contains only one parameter, then you can omit the tuple.\
/// (In current ohkami, at most *2* path params can be handled.)
/// 
/// <br>
/// 
/// ```
/// use ohkami::prelude::*;
/// 
/// struct MyParam;
/// impl<'p> ohkami::FromParam<'p> for MyParam {
///     type Error = std::convert::Infallible;
///     fn from_param(param: std::borrow::Cow<'p, str>) -> Result<Self, Self::Error> {
///         Ok(MyParam)
///     }
/// }
/// 
/// async fn handler_1(param: (MyParam,)) -> Response {
///     todo!()
/// }
/// 
/// async fn handler_2(str_param: &str) -> Response {
///     todo!()
/// }
/// ```
#[cfg_attr(feature="testing", derive(Clone))]
pub struct Ohkami<F: FangProc> {
    pub(crate) routes: TrieRouter,

    /// apply just before merged to another or called `howl`
    pub(crate) fangs:  Arc<dyn Fangs<F>>,
}

trait FangsObject<Inner: FangProc> {
    fn build(self, inner: Inner) -> Arc<dyn FangProcObject>;
}
trait FangProcObject {
    fn bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>>;
}
const _: () = {
    impl<
        Inner: FangProc,
        Fs: Fangs<Inner>,
    > FangsObject<Inner> for Fs {
        fn build(self, inner: Inner) -> Arc<dyn FangProcObject> {
            
        }
    }
};

impl Ohkami {
    /// Create new `Ohkami` on the routing.
    /// 
    /// ---
    ///
    /// `routes` is a routing item or a tuple of them :
    /// 
    /// ```
    /// # use ohkami::Route;
    /// #
    /// # async fn handler1() -> &'static str {"1"}
    /// # async fn handler2() -> &'static str {"2"}
    /// # async fn handler3() -> &'static str {"3"}
    /// #
    /// # let _ =
    /// (
    ///     "/a"
    ///         .GET(handler1)
    ///         .POST(handler2),
    ///     "/b"
    ///         .PUT(handler3),
    ///     //...
    /// )
    /// # ;
    /// ```
    /// 
    /// ---
    /// 
    /// Handler is an _**async**_ function :
    /// 
    /// > `({path params}, {FromRequest values},...) -> {IntoResponse value}`
    ///
    /// `{path params}` is a `FromParam` value or a tuple of them
    pub fn new(routes: impl build::Routes) -> Self {
        let mut router = TrieRouter::new();
        routes.apply(&mut router);

        Self {
            routes: router,
            fangs:  Vec::new(),
        }
    }

    /// Create new ohkami with the fangs on the routing.
    /// 
    /// ---
    ///
    /// `fangs` is an item that implements `FrontFang` or `BackFang`, or tuple of such items
    /// 
    /// NOTE:
    /// `fangs` passed here are executed just before/after a handler in this `Ohkami` called for a request.
    /// If you'd like to call some fangs for any requests, give them to `.howl_with()`!
    /// 
    /// ```
    /// use ohkami::prelude::*;
    /// 
    /// struct Auth;
    /// impl FrontFang for Auth {
    ///     type Error = Response;
    ///     async fn bite(&self, req: &mut Request) -> Result<(), Self::Error> {
    ///         Ok(())
    ///     }
    /// }
    /// 
    /// # async fn handler1() -> &'static str {"1"}
    /// # async fn handler2() -> &'static str {"2"}
    /// # async fn handler3() -> &'static str {"3"}
    /// #
    /// # let _ =
    /// Ohkami::with(Auth, (
    ///     "/a"
    ///         .GET(handler1)
    ///         .POST(handler2),
    ///     "/b"
    ///         .PUT(handler3),
    ///     //...
    /// ))
    /// # ;
    /// ```
    pub fn with<T>(fangs: impl Fangs<T>, routes: impl build::Routes) -> Self {
        let mut router = TrieRouter::new();
        routes.apply(&mut router);

        Self {
            routes: router,
            fangs:  fangs.collect(),
        }
    }
}

impl Ohkami {
    pub(crate) fn into_router(self) -> TrieRouter {
        let mut router = self.routes;

        for (methods, fang) in self.fangs {
            router.apply_fang(methods, fang);
        }

        #[cfg(feature="DEBUG")]
        println!("{router:#?}");

        router
    }
}
