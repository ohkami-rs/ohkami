pub(crate) mod router;
mod build;
mod howl;

use router::TrieRouter;
use crate::fang::Fangs;
use crate::Method;


/// # Ohkami - a robust wolf who serves your web app
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// # use ohkami::prelude::*;
/// # use ohkami::serde::Serialize;
/// # use ohkami::typed::ResponseBody;
/// # use ohkami::typed::status::{OK, Created};
/// # 
/// 
/// struct Auth;
/// impl FrontFang for Auth {
///     /* 〜 */
/// #    async fn bite(&self, req: &mut Request) -> Result<(), Response> {
/// #        // Do something...
/// #
/// #        Ok(())
/// #    }
/// }
/// 
/// # #[ResponseBody(JSON)]
/// # #[derive(Serialize)]
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
///         "/users".
///             POST(create_user),
///         "/users/:id".
///             GET(get_user_by_id).
///             PATCH(update_user),
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
/// async ({path_params}?, {`FromRequest` type}s...) -> {`IntoResponse` type}
/// 
/// #### path_params：
/// A tuple of types that implement `FromParam` trait.\
/// `String`, `&str`, and primitive integers are splecially allowed to be used without tuple：
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
pub struct Ohkami {
    pub(crate) routes: TrieRouter,

    /// apply just before merged to another or called `howl`
    pub(crate) fangs:  Vec<(&'static [Method], crate::fang::Fang)>,
}

impl Ohkami {
    /// - `routes` is tuple of routing item :
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
    ///     "/a".
    ///         GET(handler1).
    ///         POST(handler2),
    ///     "/b".
    ///         PUT(handler3),
    ///     //...
    /// )
    /// # ;
    /// ```
    pub fn new(routes: impl build::Routes) -> Self {
        let mut router = TrieRouter::new();
        routes.apply(&mut router);

        Self {
            routes: router,
            fangs:  Vec::new(),
        }
    }

    /// - `fangs` is an item that implements `FrontFang` or `BackFang`, or tuple of such items:
    /// 
    /// ```
    /// use ohkami::prelude::*;
    /// 
    /// struct Log;
    /// impl FrontFang for Log {
    ///     async fn bite(&self, req: &mut Request) -> Result<(), Response> {
    ///         println!("{req:?}");
    ///         Ok(())
    ///     }
    /// }
    /// ```
    /// `fangs` passed here are executed just before/after a handler in this `Ohkami` is called for the request.
    /// If you use some fangs for any requests, specify them in `.howl_with`!
    /// 
    /// <br/>
    /// 
    /// - `routes` is tuple of routing item :
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
    ///     "/a".
    ///         GET(handler1).
    ///         POST(handler2),
    ///     "/b".
    ///         PUT(handler3),
    ///     //...
    /// )
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

    #[cfg(all(feature="DEBUG", test))]
    pub(crate) fn clone(&self) -> Self {
        Self {
            routes: self.routes.clone(),
            fangs:  self.fangs .clone(),
        }
    }
}
