#[cfg(test)]
mod _test;

#[cfg(feature="__rt_native__")]
mod dir;

pub(crate) mod routing;
pub use routing::{Route, Routing};

use crate::fang::Fangs;
use crate::router::base::Router;
use std::sync::Arc;

#[cfg(feature="__rt_native__")]
use crate::{__rt__, Session};

#[cfg(all(feature="__rt_native__", feature="rt_tokio", feature="tls"))]
use tokio_rustls::TlsAcceptor;

#[cfg(all(feature="__rt_native__", feature="rt_tokio", feature="tls"))]
use crate::tls::TlsStream;

/// # Ohkami - a smart wolf who serves your web app
/// 
/// ## Definition
/// 
/// See [`Ohkami::new`] for more detail.
/// 
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
/// # use ohkami::serde::Serialize;
/// # use ohkami::typed::status::{OK, Created};
/// # use ohkami::format::JSON;
/// # use ohkami::{Fang, FangProc};
/// 
/// struct Auth;
/// impl<I: FangProc> Fang<I> for Auth {
///     /* 〜 */
/// #   type Proc = AuthProc;
/// #   fn chain(&self, inner: I) -> Self::Proc {
/// #       AuthProc
/// #   }
/// # }
/// # struct AuthProc;
/// # impl FangProc for AuthProc {
/// #     async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
/// #         Response::NotImplemented()
/// #     }
/// # }
/// 
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
/// # async fn create_user() -> Created<JSON<User>> {
/// #     Created(JSON(User {
/// #         id:   42,
/// #         name: String::from("ohkami"),
/// #         age:  None,
/// #     }))
/// # }
/// # 
/// # async fn get_user_by_id(id: usize) -> Result<JSON<User>, APIError> {
/// #     Ok(JSON(User {
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
///     let api_ohkami = Ohkami::new((
///         Auth,
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
/// ### handler signature
/// 
/// `async ({path params}?, {FromRequest<'_> type}s...) -> {IntoResponse type}`
/// 
/// - handler itself must be `Send` + `Sync` + 'static
/// - returned `Future` must be `Send` + 'static
/// 
/// excpet for `rt_worker`, where `Send` or `Sync` bound is not required.
/// 
/// ### path params
/// 
/// A tuple of types that implement `FromParam` trait e.g. `(&str, usize)`.\
/// If the path contains only one parameter, then you can omit the tuple \
/// e.g. just `param: &str`.\
/// (Current ohkami handles at most *2* path params.)
/// 
/// <br>
/// 
/// ## Serving
/// 
/// ### native async runtimes
/// 
/// `.howl(address)` to start serving.
/// 
/// ```no_run
/// # fn my_ohkami() -> ohkami::Ohkami {ohkami::Ohkami::new(())}
/// # 
/// #[tokio::main]
/// async fn main() {
///     let o = my_ohkami();
///     
///     o.howl("localhost:5000").await
/// }
/// ```
/// 
/// ### worker ( Cloudflare Workers )
/// 
/// - [worker](https://crates.io/crates/worker)
/// 
/// `#[ohkami::worker]` ( async ) fn returning `Ohkami` is the
/// Worker definition.
/// 
/// ```ignore
/// # fn my_ohkami() -> ohkami::Ohkami {ohkami::Ohkami::new(())}
/// # 
/// #[ohkami::worker]
/// async fn worker() -> Ohkami {
///     my_ohkami()
/// }
/// ```
/// 
/// ### lambda ( AWS Lambda )
/// 
/// - [lambda_runtime](https://crates.io/crates/lambda_runtime) ( and [tokio](https://crates.io/crates/tokio) )
/// 
/// Pass to `lambda_runtime::run` is the way to let your `Ohkami`
/// work on AWS Lambda.
/// 
/// ```ignore
/// # fn my_ohkami() -> ohkami::Ohkami {ohkami::Ohkami::new(())}
/// # 
/// #[tokio::main]
/// async fn main() -> Result<(), lambda_runtime::Error> {
///     lambda_runtime::run(my_ohkami()).await
/// }
/// ```
/// 
/// <br>
/// 
/// ## Testing
/// 
/// Ohkami support **no-network** testing.
/// 
/// Currently this may not work on `rt_worker`. Consider generating HTTP client
/// types with `openapi` feature and performaing tests using the client with Ohkami
/// running by `npm run dev`.
/// 
/// ```
/// use ohkami::prelude::*;
/// # 
/// # fn my_ohkami() -> ohkami::Ohkami {
/// #     ohkami::Ohkami::new(())
/// # }
/// 
/// #[cfg(test)]
/// #[tokio::test]
/// async fn test_ohkami() {
///     use ohkami::testing::*; // <--
///     
///     let t = my_ohkami()
///         .test(); // <--
///     
///     {
///         let req = TestRequest::GET("/");
///         let res = t.oneshot(req).await;
///         assert_eq!(res.status(), Status::OK);
///         assert_eq!(res.text(), Some("Hello, world!"));
///     }
///     {
///         let req = TestRequest::POST("/");
///         let res = t.oneshot(req).await;
///         assert_eq!(res.status(), Status::NotFound);
///     }
/// }
/// ```
/// 
/// <br>
/// 
/// ## Generics DI
/// 
/// A way of DI -*Dependency Injection*- is **generics** :
/// 
/// ```
/// use ohkami::prelude::*;
/// 
/// # //////////////////////////////////////////////////////////////////////
/// # /// errors
/// # 
/// # enum MyError {
/// #     Sqlx(sqlx::Error),
/// # }
/// # impl IntoResponse for MyError {
/// #     fn into_response(self) -> Response {
/// #         match self {
/// #             Self::Sqlx(e) => Response::InternalServerError(),
/// #         }
/// #     }
/// # }
/// # 
/// //////////////////////////////////////////////////////////////////////
/// /// repository
/// 
/// trait Repository: Send + Sync + 'static {
///     fn get_user_name_by_id(
///         &self,
///         id: i64,
///     ) -> impl Future<Output = Result<String, MyError>> + Send;
/// }
/// 
/// struct PostgresRepository(sqlx::PgPool);
/// impl Repository for PostgresRepository {
///     async fn get_user_name_by_id(&self, id: i64) -> Result<String, MyError> {
///         let sql = r#"
///             SELECT name FROM users WHERE id = $1
///         "#;
///         sqlx::query_scalar::<_, String>(sql)
///             .bind(id)
///             .fetch_one(&self.0)
///             .await
///             .map_err(MyError::Sqlx)
///     }
/// }
/// 
/// //////////////////////////////////////////////////////////////////////
/// /// routes
/// 
/// #[derive(Serialize)]
/// struct User {
///     id: u32,
///     name: String,
/// }
/// 
/// async fn get_user<R: Repository>(
///     id: u32,
///     Context(r): Context<'_, R>,
/// ) -> Result<JSON<User>, MyError> {
///     let user_name = r.get_user_name_by_id(id as i64).await?;
/// 
///     Ok(JSON(User {
///         id: id as u32,
///         name: user_name,
///     }))
/// }
/// 
/// fn users_ohkami<R: Repository>() -> Ohkami {
///     Ohkami::new((
///         "/:id".GET(get_user::<R>),
///     ))
/// }
/// ```
pub struct Ohkami {
    router: Router,
    /// apply just before merged to another, or just before `howl`ing
    fangs:  Option<Arc<dyn Fangs>>,
}

impl Ohkami {
    /// Create Ohkami by the routing
    /// 
    /// ### route
    /// 
    /// - `/`
    /// - `(/:?[a-zA-Z0-9_\-\.]+)+`
    /// 
    /// Segments starting with `:` defines *path params*.
    /// 
    /// ### routing
    /// 
    /// A tuple like
    /// 
    /// ```
    /// use ohkami::Route;
    /// 
    /// # use ohkami::fang::FangAction;
    /// # #[derive(Clone)] struct Logger;
    /// # impl FangAction for Logger {}
    /// # #[derive(Clone)] struct Auth;
    /// # impl FangAction for Auth {}
    /// # async fn get_handler() {}
    /// # async fn put_handler() {}
    /// # async fn post_handler() {}
    /// # 
    /// # let _ =
    /// (
    ///     // 0 or more `Fang`s of this Ohkami
    ///     Logger,
    ///     Auth,
    ///     
    ///     // 0 or more handler routes
    ///     "/route1"
    ///         .GET(get_handler)
    ///         .PUT(put_handler),
    ///     "/route2/:param"
    ///         .POST(post_handler),
    /// )
    /// # ;
    /// ```
    /// 
    /// ### handler signature
    /// 
    /// `async ({path params}?, {FromRequest<'_> type}s...) -> {IntoResponse type}`
    /// 
    /// On native runtimes or `rt_lambda`,
    /// 
    /// - handler itself must be `Send` + `Sync`
    /// - returned `Future` must be `Send`
    /// 
    /// ### path params
    /// 
    /// A tuple of types that implement `FromParam` trait e.g. `(&str, usize)`.\
    /// If the path contains only one parameter, then you can omit the tuple \
    /// e.g. just `param: &str`.\
    /// (Current ohkami handles at most *2* path params.)
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
    /// async fn handler_2(param: &str) -> Response {
    ///     todo!()
    /// }
    /// ```
    /// 
    /// ### nesting
    /// 
    /// `Route::By` creates a nested route:
    /// 
    /// ```
    /// use ohkami::{Ohkami, Route};
    /// 
    /// # fn __() -> Ohkami {
    /// # let another_ohkami = Ohkami::new(());
    /// Ohkami::new(
    ///     "/route".By(another_ohkami),
    /// )
    /// # }
    /// ```
    /// 
    /// ### static directory serving
    /// 
    /// `.Dir` mounts a directory and serves all files in it/its sub directories.
    /// 
    /// This doesn't work on `rt_worker` ( of course because there Ohkami can't
    /// touch your local file system ). Consider using `asset` of wrangler.{toml/json}
    /// ( or `--asset` flag of `npm run {dev/deploy}` )
    /// 
    /// ```
    /// use ohkami::{Ohkami, Route};
    /// 
    /// # fn __() -> Ohkami {
    /// # let another_ohkami = Ohkami::new(());
    /// Ohkami::new(
    ///     "/public".Dir("./path/to/dir"),
    /// )
    /// # }
    /// ```
    /// 
    /// ### note
    /// 
    /// Fangs of this `routing` tuple are *always* called when a request once
    /// comes to this `Ohkami` *independent of its method or detail path*.
    /// 
    /// If you need to apply some fangs only for a request to specific
    /// method and path, consider using *local fangs* :
    /// 
    /// ```
    /// use ohkami::{Ohkami, Route};
    /// 
    /// # #[derive(Clone)] struct Auth;
    /// # impl ohkami::fang::FangAction for Auth {}
    /// # #[derive(Clone)] struct SomeFang;
    /// # impl ohkami::fang::FangAction for SomeFang {}
    /// # async fn get_user_profile() {}
    /// # let _ =
    /// Ohkami::new((
    ///     "/users/:id"
    ///         .GET((Auth, SomeFang, get_user_profile)),
    ///         // apply `Auth`, `SomeFang` only on `GET /users/:id`
    /// ))
    /// # ;
    /// ```
    pub fn new<Fangs>(routing: impl Routing<Fangs>) -> Self {
        let mut this = Self {
            router: Router::new(),
            fangs:  None,
        };
        routing.apply(&mut this);
        this
    }
    /// Create Ohkami by the fangs and routing
    /// 
    /// This is almost the same as [`Ohkami::new`](crate::Ohkami::new), but
    /// takes fangs and handler routes separately.
    pub fn with(fangs: impl Fangs + 'static, routes: impl Routing) -> Self {
        let mut this = Self {
            router: Router::new(),
            fangs:  Some(Arc::new(fangs)),
        };
        routes.apply(&mut this);
        this
    }

    pub(crate) fn into_router(self) -> Router {
        let Self { fangs, mut router } = self;

        if let Some(fangs) = fangs {
            router.apply_fangs(router.id(), fangs);
        }

        #[cfg(feature="DEBUG")]
        println!("{router:#?}");

        router
    }

    #[cfg(feature="__rt_native__")]
    /// Bind this `Ohkami` to an address and start serving !
    /// 
    /// `bind` is：
    /// 
    /// - `tokio::net::ToSocketAddrs` item or `tokio::net::TcpListener`
    /// - `async_std::net::ToSocketAddrs` item or `async_std::net::TcpListener`
    /// - `smol::net::AsyncToSocketAddrs` item or `smol::net::TcpListener`
    /// - `std::net::ToSocketAddrs` item or `{glommio, nio}::net::TcpListener`
    /// 
    /// for each async runtime.
    /// 
    /// *note* : Keep-Alive timeout is 42 seconds by default.
    /// This is configureable by `OHKAMI_KEEPALIVE_TIMEOUT`
    /// environment variable.
    /// 
    /// <br>
    /// 
    /// *example.rs*
    /// ```no_run
    /// use ohkami::prelude::*;
    /// use ohkami::typed::status;
    /// 
    /// async fn hello() -> &'static str {
    ///     "Hello, ohkami!"
    /// }
    /// 
    /// async fn health_check() -> status::NoContent {
    ///     status::NoContent
    /// }
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     Ohkami::new((
    ///         "/".GET(hello),
    ///         "/healthz".GET(health_check),
    ///     )).howl("localhost:5000").await
    /// }
    /// ```
    /// 
    /// ---
    /// 
    /// *example_glommio.rs*
    /// ```ignore
    /// use ohkami::prelude::*;
    /// use ohkami::util::num_cpus;
    /// use glommio::{LocalExecutorPoolBuilder, PoolPlacement, CpuSet};
    /// 
    /// async fn hello() -> &'static str {
    ///     "Hello, ohkami!"
    /// }
    /// 
    /// fn main() {
    ///     LocalExecutorPoolBuilder::new(PoolPlacement::MaxSpread(
    ///         num_cpus::get(), CpuSet::online().ok()
    ///     )).on_all_shards(|| {
    ///         Ohkami::new((
    ///             "/user/:id"
    ///                 .GET(echo_id),
    ///         )).howl("0.0.0.0:3000")
    ///     }).unwrap().join_all();
    /// }
    /// ```
    /// 
    /// ---
    /// 
    /// *example_with_tcp_listener.rs*
    /// ```no_run
    /// use ohkami::prelude::*;
    /// use tokio::net::TcpSocket; // <---
    /// 
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     let socket = TcpSocket::new_v4()?;
    /// 
    ///     socket.bind("0.0.0.0:5000".parse().unwrap())?;
    ///
    ///     let listener = socket.listen(1024)?;
    /// 
    ///     Ohkami::new((
    ///         "/".GET(async || {
    ///             "Hello, TcpListener!"
    ///         }),
    ///     )).howl(listener).await;
    /// 
    ///     Ok(())
    /// }
    /// ```
    pub async fn howl<T>(self, bind: impl __rt__::IntoTcpListener<T>) {
        let (router, _) = self.into_router().finalize();
        let router = Arc::new(router);

        let listener = bind.ino_tcp_listener().await;

        let (wg, ctrl_c) = (sync::WaitGroup::new(), sync::CtrlC::new());

        while let Some(accept) = ctrl_c.until_interrupt(listener.accept()).await {
            let (connection, addr) = {
                #[cfg(any(feature="rt_tokio", feature="rt_smol", feature="rt_nio"))] {
                    let Ok((connection, addr)) = accept else {continue};
                    (connection, addr)
                }
                #[cfg(any(feature="rt_glommio"))] {
                    let Ok(connection) = accept else {continue};
                    let Ok(addr) = connection.peer_addr() else {continue};
                    (connection, addr)
                }
            };

            let session = Session::new(
                router.clone(),
                connection,
                addr.ip()
            );

            let wg = wg.add();
            __rt__::spawn(async move {
                session.manage().await;
                wg.done();
            });
        }

        crate::INFO!("interrupted, trying graceful shutdown...");
        drop(listener);

        crate::INFO!("waiting {} session(s) to finish...", wg.count());
        wg.await;
    }

    #[cfg(all(feature="__rt_native__", feature="rt_tokio", feature="tls"))]
    /// Bind this `Ohkami` to an address and start serving with TLS/HTTPS support!
    /// 
    /// This method works like `howl` but upgrades connections to HTTPS using the provided
    /// rustls configuration. This functionality is only available with the `rt_tokio` feature.
    /// 
    /// ### Parameters
    /// 
    /// - `bind`: Same as `howl`, can be a socket address or TcpListener
    /// - `tls_config`: A rustls server configuration containing your certificates and keys
    /// 
    /// ### Example
    /// 
    /// ```no_run
    /// use ohkami::prelude::*;
    /// use rustls::ServerConfig;
    /// use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    /// 
    /// use std::fs::File;
    /// use std::io::BufReader;
    /// 
    /// async fn hello() -> &'static str {
    ///     "Hello, secure ohkami!"
    /// }
    /// 
    /// #[tokio::main]
    /// async fn main() -> std::io::Result<()> {
    ///     // Initialize rustls crypto provider
    ///     rustls::crypto::ring::default_provider().install_default()
    ///         .expect("Failed to install rustls crypto provider");
    /// 
    ///     // Load certificates and private key
    ///     let cert_file = File::open("server.crt")?;
    ///     let key_file = File::open("server.key")?;
    ///     
    ///     let cert_chain = rustls_pemfile::certs(&mut BufReader::new(cert_file))
    ///         .map(|cd| cd.map(CertificateDer::from))
    ///         .collect::<Result<Vec<_>, _>>()?;
    ///     
    ///     let key = rustls_pemfile::read_one(&mut BufReader::new(key_file))?
    ///         .map(|p| match p {
    ///             rustls_pemfile::Item::Pkcs1Key(k) => PrivateKeyDer::Pkcs1(k),
    ///             rustls_pemfile::Item::Pkcs8Key(k) => PrivateKeyDer::Pkcs8(k),
    ///             _ => panic!("Unexpected private key type"),
    ///         })
    ///         .expect("Failed to read private key");
    /// 
    ///     // Build TLS configuration
    ///     let tls_config = ServerConfig::builder()
    ///         .with_no_client_auth()
    ///         .with_single_cert(cert_chain, key)
    ///         .expect("Failed to build TLS configuration");
    /// 
    ///     // Create and run Ohkami with HTTPS
    ///     Ohkami::new((
    ///         "/".GET(hello),
    ///     )).howls("0.0.0.0:8443", tls_config).await;
    ///     
    ///     Ok(())
    /// }
    /// ```
    /// 
    /// ```sh
    /// $ openssl req -x509 -newkey rsa:4096 -nodes -keyout server.key -out server.crt -days 365 -subj "/CN=localhost"
    /// 
    /// $ cargo run
    /// ```
    /// 
    /// ```sh
    /// $ curl --insecure https://localhost:8443
    /// Hello, secure ohkami!
    /// ```
    pub async fn howls<T>(self, bind: impl __rt__::IntoTcpListener<T>, tls_config: rustls::ServerConfig) {    
        let (router, _) = self.into_router().finalize();
        let router = Arc::new(router);
        let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));
    
        let listener = bind.ino_tcp_listener().await;
        let (wg, ctrl_c) = (sync::WaitGroup::new(), sync::CtrlC::new());
    
        while let Some(accept) = ctrl_c.until_interrupt(listener.accept()).await {
            let Ok((tcp_stream, addr)) = accept else { continue };
            
            let Ok(tls_stream) = tls_acceptor.accept(tcp_stream).await else { continue };
            
            let session = Session::new(
                router.clone(),
                TlsStream(tls_stream),
                addr.ip()
            );
    
            let wg = wg.add();
            tokio::spawn(async move {
                session.manage().await;
                wg.done();
            });
        }
    
        crate::INFO!("interrupted, trying graceful shutdown...");
        drop(listener);
    
        crate::INFO!("waiting {} session(s) to finish...", wg.count());
        wg.await;
    }

    #[cfg(feature="rt_worker")]
    #[doc(hidden)]
    pub async fn __worker__(self,
        req: ::worker::Request,
        env: ::worker::Env,
        ctx: ::worker::Context,
    ) -> ::worker::Response {
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Called `#[ohkami::worker]`; req: {req:?}");

        let mut ohkami_req = crate::Request::init();
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `ohkami::Request::init`");

        let mut ohkami_req = unsafe {std::pin::Pin::new_unchecked(&mut ohkami_req)};
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Put request in `Pin`");

        let take_over = ohkami_req.as_mut().take_over(req, env, ctx).await;
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `ohkami::Request::take_over`: {ohkami_req:?}");

        let ohkami_res = match take_over {
            Ok(()) => {#[cfg(feature="DEBUG")] ::worker::console_debug!("`take_over` succeed");
                let (router, _) = self.into_router().finalize();
                #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `self.router.finalize`");
                
                router.handle(&mut ohkami_req).await
            }
            Err(e) => {#[cfg(feature="DEBUG")] ::worker::console_debug!("`take_over` returned an error response: {e:?}");
                e
            }
        };
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Successfully generated ohkami::Response: {ohkami_res:?}");

        let res = ohkami_res.into();
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `ohkami::Response` --into--> `worker::Response`: {res:?}");

        res
    }

    #[cfg(feature="openapi")]
    #[cfg(feature="__rt_native__")]
    /// Generate OpenAPI document.
    /// 
    /// ### note
    ///  
    /// - Currently, only **JSON** is supported as the document format.
    /// - When the binary size matters, you should prepare a feature flag
    ///   activating `ohkami/openapi` in your package, and put all your codes
    ///   around `openapi` behind that feature via `#[cfg(feature = ...)]` or
    ///   `#[cfg_attr(feature = ...)]`.
    /// - This generates `openapi.json`. Use `generate_to` to configure the
    ///   file path.
    /// 
    /// ### example
    /// 
    /// ```no_run
    /// use ohkami::prelude::*;
    /// use ohkami::openapi::{OpenAPI, Server};
    /// 
    /// // An ordinal Ohkami definition, not special
    /// fn my_ohkami() -> Ohkami {
    ///     Ohkami::new((
    ///         "/hello"
    ///             .GET(|| async {"Hello, OpenAPI!"}),
    ///     ))
    /// }
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     let o = my_ohkami();
    /// 
    ///     // Here generating openapi.json
    ///     o.generate(OpenAPI {
    ///         title: "Sample API",
    ///         version: "0.1.9",
    ///         servers: &[
    ///             Server::at("http://api.example.com/v1")
    ///                 .description("Main (production) server"),
    ///             Server::at("http://staging-api.example.com")
    ///                 .description("Internal staging server for testing")
    ///         ]
    ///      });
    /// 
    ///     o.howl("localhost:5000").await
    /// }
    /// ```
    pub fn generate(&self, metadata: crate::openapi::OpenAPI) {
        self.generate_to("openapi.json", metadata)
    }

    #[cfg(feature="openapi")]
    #[cfg(feature="__rt_native__")]
    pub fn generate_to(&self, file_path: impl AsRef<std::path::Path>, metadata: crate::openapi::OpenAPI) {
        let file_path = file_path.as_ref();
        std::fs::write(file_path, self.__openapi_document_bytes__(metadata))
            .expect(&format!("failed to write OpenAPI document JSON to {}", file_path.display()))
    }

    #[cfg(feature="openapi")]
    #[doc(hidden)]
    pub fn __openapi_document_bytes__(&self, openapi: crate::openapi::OpenAPI) -> Vec<u8> {
        let (router, routes) = (Self {
            router: self.router.clone(),
            fangs:  self.fangs.clone()
        }).into_router().finalize();

        crate::DEBUG!("[openapi_document_bytes] routes = {routes:#?}, router = {router:#?}");

        let doc = router.gen_openapi_doc(
            routes.iter().map(|(r, map)| (&**r, map.keys().copied())),
            openapi
        );

        let mut bytes = ::serde_json::to_vec_pretty(&doc).expect("failed to serialize OpenAPI document");
        bytes.push(b'\n');

        bytes
    }
}

const _: () = {
    #[cfg(feature="rt_lambda")]
    static ROUTER: std::sync::OnceLock<crate::router::r#final::Router> = std::sync::OnceLock::new();

    #[cfg(feature="rt_lambda")]
    impl lambda_runtime::Service<
        lambda_runtime::LambdaEvent<
            crate::x_lambda::LambdaHTTPRequest
        >
    > for Ohkami {
        type Response = lambda_runtime::FunctionResponse<
            crate::x_lambda::LambdaResponse,
            std::pin::Pin<Box<dyn ohkami_lib::Stream<Item = Result<String, std::convert::Infallible>> + Send>>
        >;
        type Error = lambda_runtime::Error;

        #[cfg(feature="nightly")]
        type Future = impl std::future::Future<Output = Result<Self::Response, Self::Error>>;
        #[cfg(not(feature="nightly"))]
        type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

        fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
            if ROUTER.get().is_none() {                
                let o = std::mem::replace(self, Ohkami::new(()));
                let (router, _) = o.into_router().finalize();
                
                ROUTER.set(router).ok().expect("`ROUTER.set()` was called more than once for an `Ohkami` instance");
            }

            std::task::Poll::Ready(Ok(()))
        }

        fn call(
            &mut self,
            req: lambda_runtime::LambdaEvent<crate::x_lambda::LambdaHTTPRequest>
        ) -> Self::Future {
            let f = async move {
                let mut ohkami_req = crate::Request::init();
                let mut ohkami_req = unsafe {std::pin::Pin::new_unchecked(&mut ohkami_req)};
                ohkami_req.as_mut().take_over(req)?;

                let mut ohkami_res = ROUTER.get().unwrap().handle(&mut ohkami_req).await;
                ohkami_res.complete();

                Ok(ohkami_res.into())
            };

            #[cfg(feature="nightly")] {f}
            #[cfg(not(feature="nightly"))] {Box::pin(f)}
        }
    }
};

#[cfg(feature="__rt_native__")]
mod sync {
    pub struct WaitGroup(std::ptr::NonNull<
        std::sync::atomic::AtomicUsize
    >);
    const _: () = {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::ptr::NonNull;
        use std::future::Future;
        use std::task::{Context, Poll};
        use std::pin::Pin;

        unsafe impl Send for WaitGroup {}
        unsafe impl Sync for WaitGroup {}

        impl WaitGroup {
            pub fn new() -> Self {
                let n = AtomicUsize::new(0);
                let n = Box::leak(Box::new(n));
                Self(NonNull::new(n).unwrap())
            }

            pub fn count(&self) -> usize {
                unsafe {self.0.as_ref()}.load(Ordering::Relaxed)
            }

            #[inline]
            pub fn add(&self) -> Self {
                let ptr = self.0;
                unsafe {ptr.as_ref()}.fetch_add(1, Ordering::Relaxed);
                Self(ptr)
            }

            pub fn done(self) {
                /* just drop */
            }
        }

        impl Drop for WaitGroup {
            #[inline]
            fn drop(&mut self) {
                unsafe {self.0.as_ref()}.fetch_sub(1, Ordering::Release);
            }
        }

        impl Future for WaitGroup {
            type Output = ();
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                if unsafe {self.0.as_ref()}.load(Ordering::Acquire) == 0 {
                    crate::DEBUG!("[WaitGroup::poll] Ready");
                    Poll::Ready(())
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }
    };

    pub struct CtrlC { index: usize }
    const _: () = {
        use std::sync::atomic::{AtomicBool, AtomicUsize, AtomicPtr, Ordering};
        use std::task::{Context, Poll, Waker};
        use std::pin::Pin;
        use std::ptr::null_mut;

        static INTERRUPTED: AtomicBool = AtomicBool::new(false);

        /* write access is used when pushing initial (null) AtomicPtr in `Ctrlc::new` */
        static WAKERS: std::sync::RwLock<Vec<AtomicPtr<Waker>>> = std::sync::RwLock::new(Vec::new());

        impl CtrlC {
            pub fn new() -> Self {
                /*
                    When finally get Ctrl-C signal, let's set `INTERRUPTED` to true and
                    wake all wakers for Ohkamis on one or more threads.

                    This is intended to work correctly in both :

                    1. A single Ohkami is running on multi-thread async runtime.
                    2. Spawning some threads and single-thread Ohkami is running on
                       each thread with single-thread async runtime.
                       - glommio is designed to do so
                       - even in other runtimes, sometimes this way of entrypoint
                         with `SO_REUSEADDR` may work in better performance than ordinary
                         one with multi-thread runtime.

                    For case 1., we only have to hold the single `AtomicPtr<Waker>`
                    corresponded to the Ohkami in `static WAKER`, and here retrieve/wake it :

                    ```
                    ::ctrlc::set_handler(|| {
                        INTERRUPTED.store(true, Ordering::SeqCst);
                        let waker = WAKER.swap(null_mut(), Ordering::SeqCst);
                        if !waker.is_null() {
                            unsafe {Box::from_raw(waker)}.wake();
                        }
                    }).expect("Something went wrong with Ctrl-C");

                    ```

                    But taking case 2. into consideration, we must terminate other threads
                    together with the main thread in this handler. So we have to hold
                    all `Waker`s in `WAKERS` and wake each them.
                */
                ::ctrlc::set_handler(|| {
                    INTERRUPTED.store(true, Ordering::SeqCst);

                    let wakers = WAKERS.read().unwrap();
                    crate::DEBUG!("CtrlC handler: Waiting for {} Ohkami(s)", wakers.len());
                    for w in &*wakers {
                        let w = w.swap(null_mut(), Ordering::SeqCst);
                        if !w.is_null() {
                            (unsafe {Box::from_raw(w)}).wake();
                        }
                    }
                }).ok();

                let index = {
                    static WAKER_INDEX: AtomicUsize = AtomicUsize::new(0);
                    WAKER_INDEX.fetch_add(1, Ordering::Relaxed)
                };

                #[cfg(debug_assertions)] {
                    assert_eq!(index, WAKERS.read().unwrap().len());
                }

                /* ensure that `WAKERS` has the same numbers of `Waker`s as `CtrlC` instances */
                WAKERS.write().unwrap().push(AtomicPtr::new(null_mut()));

                Self { index }
            }

            pub fn until_interrupt<T>(&self, task: impl Future<Output = T>) -> impl Future<Output = Option<T>> {
                return UntilInterrupt { index: self.index, task };

                struct UntilInterrupt<F: Future> {
                    index: usize,
                    task:  F,
                }
                impl<F: Future> Future for UntilInterrupt<F> {
                    type Output = Option<F::Output>;

                    #[inline]
                    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                        let UntilInterrupt { index, task } = unsafe {self.get_unchecked_mut()};

                        match unsafe {Pin::new_unchecked(task)}.poll(cx) {
                            Poll::Ready(t) => Poll::Ready(Some(t)),
                            Poll::Pending => match INTERRUPTED.load(Ordering::SeqCst) {
                                true => {
                                    crate::DEBUG!("[CtrlC::catch] Ready");
                                    Poll::Ready(None)
                                }
                                false => {
                                    let prev_waker = WAKERS.read().unwrap()[*index].swap(
                                        Box::into_raw(Box::new(cx.waker().clone())),
                                        Ordering::SeqCst
                                    );
                                    if !prev_waker.is_null() {
                                        unsafe {prev_waker.drop_in_place()}
                                    }
                                    Poll::Pending
                                }
                            }
                        }
                    }
                }
            }
        }
    };
}

#[cfg(all(debug_assertions, feature="__rt_native__"))]
#[cfg(test)]
mod test {
    use super::*;

    #[test] fn can_howl_on_any_native_async_runtime() {
        __rt__::testing::block_on(async {
            crate::util::timeout_in(
                std::time::Duration::from_secs(3),
                Ohkami::new(()).howl(("localhost", __rt__::testing::PORT))
            ).await
        });
    }

    #[test] fn ohkami_is_send_sync_static_on_native() {
        fn is_send_sync_static<T: Send + Sync + 'static>(_: T) {}

        let o = Ohkami::new((
            crate::fang::Context::new(String::from("resource")),
            "/".GET(async || {"Hello, world!"})
        ));

        is_send_sync_static(o);
    }
}
