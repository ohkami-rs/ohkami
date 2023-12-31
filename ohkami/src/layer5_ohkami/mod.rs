mod with_fangs; pub use with_fangs::{IntoFang};
mod build;
mod howl;

use crate::{
    layer0_lib::Method,
    layer4_router::TrieRouter,
};


/// <br/>
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
/// 
/// struct Auth;
/// impl IntoFang for Auth {
///     fn into_fang(self) -> Fang {
///         Fang(|c: &mut Context, req: &mut Request| {
///             // Do something...
/// 
///             Ok(())
///         })
///     }
/// }
/// 
/// async fn health_check(c: Context) -> Response {
///     todo!()
/// }
/// 
/// async fn create_user(c: Context) -> Response {
///     todo!()
/// }
/// 
/// async fn get_user_by_id(c: Context) -> Response {
///     todo!()
/// }
/// 
/// async fn update_user(c: Context) -> Response {
///     todo!()
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
/// - `Fn(&Response)`
/// - `Fn(Response) -> Response`
/// 
/// #### To make *front fang*：
/// - `Fn( {&/&mut Context} )`
/// - `Fn( {&/&mut Request} )`
/// - `Fn( {&/&mut Context}, {&/&mut Request} )`
/// - `_ -> Result<(), Response>` version of them
/// 
/// ## handler schema
/// - async (`Context`) -> `Response`
/// - async (`Context`, {path_params}) -> `Response`
/// - async (`Context`, {`FromRequest` values...}) -> `Response`
/// - async (`Context`, {path_params}, {`FromRequest` values...}) -> `Response`
/// 
/// path_param：A type that impls `FromParam`, or a tuple of `FromParam` types
/// 
pub struct Ohkami {
    pub(crate) routes: TrieRouter,

    /// apply just before merged to another or called `howl`
    pub(crate) fangs:  Vec<(&'static [Method], crate::layer3_fang_handler::Fang)>,
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
