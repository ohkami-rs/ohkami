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
use crate::{__rt__, session};

/// # Ohkami - a smart wolf who serves your web app
/// 
/// ## Definition
/// 
/// See [`Ohkami::new`] for more detail.
/// 
/// *example.rs*
/// ```
/// use ohkami::{fang::FangAction, IntoResponse, Response, Request};
/// # use ohkami::serde::Serialize;
/// # use ohkami::claw::{status, Path, Json};
/// 
/// // custom fangs
/// #[derive(Clone)]
/// struct Auth;
/// impl FangAction for Auth {
///     async fn fore<'b>(&'b self, req: &'b mut Request) -> Result<(), Response> {
///         Err(Response::NotImplemented())
///     }
/// }
/// 
/// // custom error
/// enum ApiError {
///     UserNotFound,
/// }
/// impl IntoResponse for ApiError {
///     fn into_response(self) -> Response {
///         match self {
///             Self::UserNotFound => Response::InternalServerError()
///         }
///     }
/// }
/// 
/// # #[derive(Serialize)]
/// # struct User {
/// #     id:   usize,
/// #     name: String,
/// #     age:  Option<usize>,
/// # }
/// # 
/// # async fn health_check() -> status::NoContent {
/// #     status::NoContent
/// # }
/// # 
/// # async fn create_user() -> status::Created<Json<User>> {
/// #     status::Created(Json(User {
/// #         id:   42,
/// #         name: String::from("ohkami"),
/// #         age:  None,
/// #     }))
/// # }
/// # 
/// # async fn get_user_by_id(Path(id): Path<usize>) -> Result<Json<User>, ApiError> {
/// #     Ok(Json(User {
/// #         id,
/// #         name: String::from("ohkami"),
/// #         age:  Some(2),
/// #     }))
/// # }
/// # 
/// # async fn update_user(Path(id): Path<usize>) -> status::NoContent {
/// #     status::NoContent
/// # }
/// 
/// // Ohkami definition
/// 
/// use ohkami::{Ohkami, Route};
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
///         "/hc".GET(health_check),
///         "/api".By(api_ohkami),
///     ))
/// }
/// ```
/// 
/// <br>
/// 
/// ### handler signature
/// 
/// `async ({FromRequest<'_> type}s...) -> {IntoResponse type}`
/// 
/// - handler itself must be `Send` + `Sync` + 'static
/// - returned `Future` must be `Send` + 'static
/// 
/// excpet for `rt_worker`, where `Send` or `Sync` bound is not required.
/// 
/// For example:
/// 
/// ```
/// # enum ApiError {}
/// # impl ohkami::IntoResponse for ApiError {
/// #    fn into_response(self) -> ohkami::Response {todo!()}
/// # }
/// 
/// use ohkami::serde::Serialize;
/// use ohkami::claw::{status, Path, Json};
/// 
/// #[derive(Serialize)]
/// struct User {
///     id:   usize,
///     name: String,
///     age:  Option<usize>,
/// }
/// 
/// async fn health_check() -> status::NoContent {
///     status::NoContent
/// }
/// 
/// async fn create_user() -> status::Created<Json<User>> {
///     status::Created(Json(User {
///         id:   42,
///         name: String::from("ohkami"),
///         age:  None,
///     }))
/// }
/// 
/// async fn get_user_by_id(
///     Path(id): Path<usize>
/// ) -> Result<Json<User>, ApiError> {
///     Ok(Json(User {
///         id,
///         name: String::from("ohkami"),
///         age:  Some(2),
///     }))
/// }
/// 
/// async fn update_user(Path(id): Path<usize>) -> status::NoContent {
///     status::NoContent
/// }
/// 
/// # /// assert `IntoHandler` impl
/// # fn __assert__() {
/// #     fn assert_impl_into_handler<T, H: ohkami::fang::handler::IntoHandler<T>>(h: H) {}
/// #     assert_impl_into_handler(health_check);
/// #     assert_impl_into_handler(create_user);
/// #     assert_impl_into_handler(get_user_by_id);
/// #     assert_impl_into_handler(update_user);
/// # }
/// ```
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
/// A way of DI is **generics** :
/// 
/// ```no_run
/// use ohkami::{Ohkami, Route};
/// use ohkami::claw::{Path, Json};
/// use ohkami::fang::Context;
/// use ohkami::serde::Serialize;
/// 
/// # use ohkami::{IntoResponse, Response};
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
/// trait UserRepository: Send + Sync + 'static {
///     fn get_user_name_by_id(
///         &self,
///         id: i64,
///     ) -> impl Future<Output = Result<String, MyError>> + Send;
/// }
/// 
/// #[derive(Clone)]
/// struct PostgresUserRepository(sqlx::PgPool);
/// impl UserRepository for PostgresUserRepository {
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
/// async fn get_user<R: UserRepository>(
///     Path(id): Path<u32>,
///     Context(r): Context<'_, R>,
/// ) -> Result<Json<User>, MyError> {
///     let user_name = r.get_user_name_by_id(id as i64).await?;
/// 
///     Ok(Json(User {
///         id: id as u32,
///         name: user_name,
///     }))
/// }
/// 
/// fn users_ohkami<R: UserRepository>() -> Ohkami {
///     Ohkami::new((
///         "/:id".GET(get_user::<R>),
///     ))
/// }
/// 
/// //////////////////////////////////////////////////////////////////////
/// /// entry point
/// 
/// #[tokio::main]
/// async fn main() {
///     let pool = sqlx::PgPool::connect("postgres://ohkami:password@localhost:5432/db")
///         .await
///         .expect("failed to connect to database");
///     
///     Ohkami::new((
///         Context::new(PostgresUserRepository(pool)),
///         "/users".By(users_ohkami::<PostgresUserRepository>()),
///     )).howl("0.0.0.0:4040").await
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
    /// `async ({FromRequest<'_> type}s...) -> {IntoResponse type}`
    /// 
    /// On native runtimes or `rt_lambda`,
    /// 
    /// - handler itself must be `Send` + `Sync`
    /// - returned `Future` must be `Send`
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
    /// `.Mount({directory_path})` mounts a directory and serves all files in it/its sub directories.
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
    ///     "/public".Mount("./path/to/dir"),
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

        crate::DEBUG!("{router:#?}");

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
    /// depending on the async runtime.
    /// 
    /// *note* : Keep-Alive timeout is 39 seconds by default.
    /// This can be configured by `OHKAMI_KEEPALIVE_TIMEOUT`
    /// environment variable.
    /// 
    /// ## Examples
    /// 
    /// ---
    /// 
    /// *example.rs*
    /// ```no_run
    /// use ohkami::{Ohkami, Route};
    /// use ohkami::claw::status;
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
    /// *example_with_tcp_listener.rs*
    /// ```no_run
    /// use ohkami::{Ohkami, Route};
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
    /// 
    /// ---
    /// 
    /// ## TLS support
    /// 
    /// `tls` feature enables TLS support:
    /// 
    /// ```toml
    /// [dependencies]
    /// ohkami = { version = "...", features = [..., "tls"] }
    /// ```
    /// 
    /// Then `howl` takes an additional parameter `tls_config`:
    /// A `rutsls::ServerConfig` containing your certificates and keys, or `None` meaning no TLS.
    /// 
    /// Example:
    /// 
    /// ```toml
    /// [dependencies]
    /// ohkami = { version = "0.24", features = ["rt_tokio", "tls"] }
    /// tokio  = { version = "1",    features = ["full"] }
    /// rustls = { version = "0.23", features = ["ring"] }
    /// rustls-pemfile = "2.2"
    /// ```
    /// 
    /// ```no_run
    /// use ohkami::{Ohkami, Route};
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
    ///     )).howl("0.0.0.0:8443", tls_config).await;
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
    /// 
    /// For localhost-testing with browser (or `curl` without `--insecure`),
    /// [`mkcert`](https://github.com/FiloSottile/mkcert) is highly recommended.
    pub async fn howl<T>(
        self,
        bind: impl __rt__::IntoTcpListener<T>,
        #[cfg(feature="tls")]
        #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
        tls_config: impl Into<Option<rustls::ServerConfig>>,
    ) {
        let (router, _) = self.into_router().finalize();
        let router = Arc::new(router);

        let listener = bind.into_tcp_listener().await;
        let (wg, ctrl_c) = (sync::WaitGroup::new(), sync::CtrlC::new());
        
        #[cfg(feature="tls")]
        let tls_acceptor = tls_config.into().map(|it| anysc_rustls::TlsAcceptor::from(Arc::new(it)));
        
        crate::INFO!("start serving on {}", listener.local_addr().unwrap());

        while let Some(accept) = ctrl_c.until_interrupt(__rt__::accept(&listener)).await {
            let Ok((connection, address)) = accept else {continue};

            #[cfg(feature="tls")]
            let connection: session::Connection = match &tls_acceptor {
                None => connection.into(),
                Some(tls_acceptor) => match ctrl_c.until_interrupt(tls_acceptor.accept(connection)).await {
                    None => break,
                    Some(Ok(tls_stream)) => tls_stream.into(),
                    Some(Err(e)) => {
                        crate::ERROR!("TLS accept error: {e}");
                        continue;
                    }
                }
            };

            let session = session::Session::new(
                connection,
                address.ip(),
                router.clone(),
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

    #[cfg(feature="rt_worker")]
    #[doc(hidden)]
    pub async fn __worker__(self,
        req: ::worker::Request,
        env: ::worker::Env,
        ctx: ::worker::Context,
    ) -> ::worker::Response {
        crate::DEBUG!("Called `#[ohkami::worker]`; req: {req:?}");

        let mut ohkami_req = crate::Request::uninit();
        crate::DEBUG!("Done `ohkami::Request::init`");

        let mut ohkami_req = std::pin::Pin::new(&mut ohkami_req);
        crate::DEBUG!("Put request in `Pin`");

        let take_over = ohkami_req.as_mut().take_over(req, env, ctx).await;
        crate::DEBUG!("Done `ohkami::Request::take_over`: {ohkami_req:?}");

        let ohkami_res = match take_over {
            Ok(()) => {
                crate::DEBUG!("`take_over` succeed");
                let (router, _) = self.into_router().finalize();
                crate::DEBUG!("Done `self.router.finalize`");
                router.handle(&mut ohkami_req).await
            }
            Err(e) => {
                crate::DEBUG!("`take_over` returned an error response: {e:?}");
                e
            }
        };
        crate::DEBUG!("Successfully generated ohkami::Response: {ohkami_res:?}");

        let res = ohkami_res.into();
        crate::DEBUG!("Done `ohkami::Response` --into--> `worker::Response`: {res:?}");

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
    /// use ohkami::{Ohkami, Route};
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
                let mut ohkami_req = crate::Request::uninit();
                let mut ohkami_req = std::pin::Pin::new(&mut ohkami_req);
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

#[cfg(feature="__rt_native__")]
#[cfg(test)]
mod test {
    use super::*;

    #[cfg(not(feature="tls"))]
    #[test]
    fn can_howl_on_any_native_async_runtime() {
        __rt__::testing::block_on(async {
            crate::util::with_timeout(
                std::time::Duration::from_secs(3),
                Ohkami::new(()).howl(("localhost", __rt__::testing::PORT))
            ).await
        });
    }
    
    #[cfg(feature="tls")]
    #[test]
    fn can_howl_with_tls_on_any_native_async_runtime() {
        let openssl_x509_newkey = |out_path: &str, keyout_path: &str| -> std::io::Result<()> {
            std::process::Command::new("openssl")
                .args([
                    "req", "-x509", "-newkey", "rsa:4096", "-nodes",
                    "-out", out_path, "-keyout", keyout_path,
                    "-days", "365", "-subj", "/CN=localhost"
                ])
                .status()
                .map(|status| status.success().then_some(()).ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to generate test certificate and key with OpenSSL"
                    )
                }))
                .flatten()
        };
        
        let is_pem_alive = |path: &std::path::Path| -> bool {
            if !path.exists() {
                return false;
            }
            
            if {
                let now = std::time::SystemTime::now();
                let created = path.metadata().unwrap().created().unwrap();
                now.duration_since(created).unwrap().as_secs() >= 60 * 60 * 24 * 365
            } {
                return false;
            }
            
            true
        };
        
        __rt__::testing::block_on(async {
            rustls::crypto::ring::default_provider().install_default().ok();
        
            let (cert_file_path, key_file_path) = {
                let target_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent().unwrap()
                    .join("target");
                (target_dir.join("test-cert.pem"), target_dir.join("test-key.pem"))
            };
            
            if !{is_pem_alive(&cert_file_path) && is_pem_alive(&key_file_path)} {
                openssl_x509_newkey(
                    cert_file_path.to_str().unwrap(),
                    key_file_path.to_str().unwrap()
                ).expect("`openssl` failed");
            }
            
            let cert_chain = rustls_pemfile::certs(&mut std::io::BufReader::new(std::fs::File::open(cert_file_path).unwrap()))
                .map(|cd| cd.map(rustls::pki_types::CertificateDer::from))
                .collect::<Result<Vec<_>, _>>()
                .expect("Failed to read certificate chain");            
            let key = rustls_pemfile::read_one(&mut std::io::BufReader::new(std::fs::File::open(key_file_path).unwrap()))
                .expect("Failed to read private key")
                .map(|p| match p {
                    rustls_pemfile::Item::Pkcs1Key(k) => rustls::pki_types::PrivateKeyDer::Pkcs1(k),
                    rustls_pemfile::Item::Pkcs8Key(k) => rustls::pki_types::PrivateKeyDer::Pkcs8(k),
                    _ => panic!("Unexpected private key type"),
                })
                .expect("Failed to read private key");
        
            let tls_config = rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(cert_chain, key)
                .expect("Failed to build TLS configuration");

            crate::util::with_timeout(
                std::time::Duration::from_secs(3),
                Ohkami::new(()).howl(("localhost", __rt__::testing::PORT), tls_config)
            ).await
        });
    }

    #[test]
    fn ohkami_is_send_sync_static_on_native() {
        fn is_send_sync_static<T: Send + Sync + 'static>(_: T) {}

        let o = Ohkami::new((
            crate::fang::Context::new(String::from("resource")),
            "/".GET(async || {"Hello, world!"})
        ));

        is_send_sync_static(o);
    }
}
