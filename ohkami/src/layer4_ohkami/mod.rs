mod with_fangs; pub use with_fangs::{IntoFang};
mod build;
mod howl;

use crate::{
    layer0_lib::Method,
    layer3_router::TrieRouter,
};


/// <br/>
/// 
/// ```
/// use ohkami::prelude::*;
/// use ohkami::serde::Serialize;
/// use ohkami::typed::ResponseBody;
/// 
/// struct Log;
/// impl IntoFang for Log {
///     fn into_fang(self) -> Fang {
///         Fang(|res: &Response| {
///             println!("{res:?}");
///         })
///     }
/// }
/// 
/// struct Auth;
/// impl IntoFang for Auth {
///     fn into_fang(self) -> Fang {
///         Fang(|req: &Request| {
///             // Do something...
/// 
///             Ok(())
///         })
///     }
/// }
/// 
/// #[ResponseBody(JSON)]
/// #[derive(Serialize)]
/// struct User {
///     id:   usize,
///     name: String,
///     age:  Option<usize>,
/// }
/// 
/// enum APIError {
///     UserNotFound
/// }
/// impl IntoResponse for APIError {
///     fn into_response(self) -> Response {
///         match self {
///             Self::UserNotFound => Response::InternalServerError()
///         }
///     }
/// }
/// 
/// async fn health_check() -> impl IntoResponse {
///     Status::NoContent
/// }
/// 
/// async fn create_user() -> Created<User> {
///     Created(User {
///         id:   42,
///         name: String::from("ohkami"),
///         age:  None,
///     })
/// }
/// 
/// async fn get_user_by_id(id: usize) -> Result<OK<User>, APIError> {
///     Ok(OK(User {
///         id,
///         name: String::from("ohkami"),
///         age:  Some(2),
///     }))
/// }
/// 
/// async fn update_user(id: usize) -> impl IntoResponse {
///     Status::OK
/// }
/// 
/// fn my_ohkami() -> Ohkami {
///     let api_ohkami = Ohkami::with((Auth, Log), (
///         "/users".
///             POST(create_user),
///         "/users/:id".
///             GET(get_user_by_id).
///             PATCH(update_user),
///     ));
/// 
///     // And, here `Log` fang of api_ohkami is duplicated with
///     // that of the root ohkami below, but it's no problem
///     // because they are merged internally.
/// 
///     Ohkami::with(Log, (
///         "/hc" .GET(health_check),
///         "/api".By(api_ohkami),
///     ))
/// }
/// ```
/// 
/// <br/>
/// 
/// ## fang schema
/// #### To make *back fang*：
/// - `Fn({&/&mut Response})`
/// - `Fn(Response) -> Response`
/// 
/// #### To make *front fang*：
/// - `Fn()`
/// - `Fn({&/&mut Request})`
/// - `_ -> Result<(), Response>` version of them
/// 
/// <br/>
/// 
/// ## handler schema
/// - async () -> `Response`
/// - async ({path_params}) -> `Response`
/// - async ({`FromRequest` values...}) -> `Response`
/// - async ({path_params}, {`FromRequest` values...}) -> `Response`
/// 
/// #### path_params：
/// A tuple of types that implement `FromParam` trait.\
/// `String`, `&str`, and primitive integers are splecially allowed to be used without tuple：
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
    pub(crate) fangs:  Vec<(&'static [Method], crate::layer2_fang_handler::Fang)>,
}

impl Ohkami {
    /// `routes` is tuple of routing item :
    /// 
    /// ```ignore
    /// "/route".
    ///     Method1(method1).
    ///     Method2(method2)
    ///     //...
    /// ```
    pub fn new(routes: impl build::Routes) -> Self {
        Self {
            routes: routes.apply(TrieRouter::new()),
            fangs:  Vec::new(),
        }
    }

    /// - `fangs` is an item that implements `IntoFang`, or tuple of such items :
    /// 
    /// ```
    /// use ohkami::prelude::*;
    /// 
    /// struct Log;
    /// impl IntoFang for Log {
    ///     fn into_fang(self) -> Fang {
    ///         Fang(|res: &Response| {
    ///             println!("{res:?}");
    ///         })
    ///     }
    /// }
    /// ```
    /// 
    /// <br/>
    /// 
    /// - `routes` is tuple of routing item :
    /// 
    /// ```ignore
    /// (
    ///     "/a".
    ///         GET(method1).
    ///         POST(method2),
    ///     "/b".
    ///         PUT(method3),
    ///     //...
    /// )
    /// ```
    pub fn with(fangs: impl with_fangs::Fangs, routes: impl build::Routes) -> Self {
        Self {
            routes: routes.apply(TrieRouter::new()),
            fangs:  fangs.collect(),
        }
    }
}

impl Ohkami {
    pub(crate) fn into_router(self) -> TrieRouter {
        let mut router = self.routes;
        for (methods, fang) in self.fangs {
            router = router.apply_fang(methods, fang)
        }
        router
    }
}
