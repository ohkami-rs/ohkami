//! Related with `ohkami/ohkami/build.rs`

#![allow(non_snake_case)]

use crate::Ohkami;
use super::{Handler, IntoHandler};
use crate::ohkami::router::RouteSections;


macro_rules! Handlers {
    ($( $method:ident ),*) => {
        pub struct Handlers {
            pub(crate) route:   RouteSections,
            $(
                pub(crate) $method: Option<Handler>,
            )*
        }
        
        impl Handlers {
            pub(crate) fn new(route_str: &'static str) -> Self {
                Self {
                    route:   RouteSections::from_literal(route_str),
                    $(
                        $method: None,
                    )*
                }
            }
        }

        impl Handlers {
            $(
                pub fn $method<T>(mut self, handler: impl IntoHandler<T>) -> Self {
                    self.$method.replace(handler.into_handler());
                    self
                }
            )*
        }
    };
} Handlers! { GET, PUT, POST, PATCH, DELETE }

pub struct ByAnother {
    pub(crate) route:  RouteSections,
    pub(crate) ohkami: Ohkami,
}

pub struct Dir {
    pub(crate) route: RouteSections,
    pub(crate) files: Vec<(
        Vec<String>,
        std::fs::File,
    )>,
} impl Dir {
    fn new(route: RouteSections, dir_path: std::path::PathBuf) -> std::io::Result<Self> {
        let dir_path = dir_path.canonicalize()?;

        if !dir_path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{} is not directory", dir_path.display()))
            )
        }

        let mut files = Vec::new(); {
            fn fetch_entries(
                dir: std::path::PathBuf
            ) -> std::io::Result<Vec<std::path::PathBuf>> {
                dir.read_dir()?
                    .map(|de| de.map(|de| de.path()))
                    .collect()
            }

            let mut entries = fetch_entries(dir_path.clone())?;
            while let Some(entry) = entries.pop() {
                if entry.is_file() {
                    if entry.starts_with(".") {
                        println!(
                            "[WARNING] `Route::Dir`: found `{}` in directory `{}`, \
                            are you sure to serve this file？",
                            entry.display(),
                            dir_path.display(),
                        )
                    }

                    files.push((
                        entry.canonicalize()?
                            .components()
                            .skip(dir_path.components().count())
                            .map(|c| c.as_os_str().to_os_string()
                                .into_string()
                                .map_err(|os_string| std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    format!("Can't read a path segment `{}`", os_string.as_encoded_bytes().escape_ascii())
                                ))
                            )
                            .collect::<std::io::Result<Vec<_>>>()?,
                        std::fs::File::open(entry)?
                    ));

                } else if entry.is_dir() {
                    entries.append(&mut fetch_entries(entry)?)

                } else {
                    continue
                }
            }
        }

        Ok(Self { route, files })
    }
}


macro_rules! Route {
    ($( $method:ident ),*) => {
        /// Core trait for ohkami's routing definition.
        /// 
        /// <br>
        /// 
        /// *example.rs*
        /// ```no_run
        /// use ohkami::{Ohkami, Route};
        /// 
        /// async fn index() -> &'static str {
        ///     "ohkami"
        /// }
        /// 
        /// async fn greet() -> &'static str {
        ///     "I'm fine."
        /// }
        /// 
        /// async fn hello() -> String {
        ///     format!("Hello!!!")
        /// }
        /// 
        /// #[tokio::main]
        /// async fn main() {
        ///     Ohkami::new((
        ///         "/"  // <-- `Route` works here...
        ///             .GET(index),
        ///         "/hello"  // <-- `Route` works here...
        ///             .GET(greet)
        ///             .PUT(hello),
        ///     )).howl("localhost:3000").await
        /// }
        /// ```
        pub trait Route {
            $(
                fn $method<T>(self, handler: impl IntoHandler<T>) -> Handlers;
            )*

            fn By(self, another: Ohkami) -> ByAnother;

            fn Dir(self, static_files_dir_path: &'static str) -> Dir;
        }

        impl Route for &'static str {
            $(
                fn $method<T>(self, handler: impl IntoHandler<T>) -> Handlers {
                    let mut handlers = Handlers::new(self);
                    handlers.$method.replace(handler.into_handler());
                    handlers
                }
            )*

            fn By(self, another: Ohkami) -> ByAnother {
                ByAnother {
                    route:  RouteSections::from_literal(self),
                    ohkami: another,
                }
            }

            fn Dir(self, path: &'static str) -> Dir {
                match Dir::new(
                    RouteSections::from_literal(self),
                    path.into()
                ) {
                    Ok(dir) => dir,
                    Err(e) => panic!("{e}")
                }
            }
        }
    };
} Route! { GET, PUT, POST, PATCH, DELETE }




#[cfg(test)] #[allow(unused)] mod __ {
    use std::borrow::Cow;
    use ::serde::{Serialize, Deserialize};
    use super::{Handlers, Route};
    use crate::{FromRequest, IntoResponse, Response, Request, Status};
    use crate::typed::Payload;
    use crate::typed::status::{OK, Created};


    enum APIError {
        DBError,
    }
    impl IntoResponse for APIError {
        fn into_response(self) -> crate::Response {
            Response::with(Status::InternalServerError)
        }
    }

    async fn health_check() -> Status {
        Status::NoContent
    }

    #[derive(Serialize)]
    struct User {
        id:       usize,
        name:     String,
        password: String,
    } const _: () = {
        impl Payload for User {
            type Type = crate::builtin::payload::JSON;
        }
    };

    mod mock {
        use super::APIError;

        pub async fn authenticate() -> Result<(), APIError> {
            Ok(())
        }

        pub const DB: __::Database = __::Database; mod __ {
            use super::APIError;

            pub struct Database;
            impl Database {
                pub async fn insert_returning_id(&self, Model: impl serde::Deserialize<'_>) -> Result<usize, APIError> {
                    Ok(42)
                }
                pub async fn update_returning_id(&self, Model: impl serde::Deserialize<'_>) -> Result<usize, APIError> {
                    Ok(24)
                }
            }
        }
    }

    #[derive(Deserialize)]
    struct CreateUser<'c> {
        name:     &'c str,
        password: &'c str,
    }
    impl Payload for CreateUser<'_> {
        type Type = crate::builtin::payload::JSON;
    }

    async fn create_user<'req>(payload: CreateUser<'req>) -> Result<Created<User>, APIError> {
        let CreateUser { name, password } = payload;

        mock::authenticate().await?;

        let id = mock::DB.insert_returning_id(CreateUser{ name, password }).await?;

        Ok(Created(User {
            id,
            name: name.to_string(),
            password: password.to_string(),
        }))
    }

    #[derive(Deserialize)]
    struct UpdateUser<'u> {
        name:     Option<&'u str>,
        password: Option<&'u str>,
    }
    impl Payload for UpdateUser<'_> {
        type Type = crate::builtin::payload::JSON;
    }

    async fn update_user<'req>(body: UpdateUser<'req>) -> Result<Status, APIError> {
        mock::authenticate().await?;
        mock::DB.update_returning_id(body).await?;

        Ok(Status::NoContent)
    }


    async fn main() {
        let _ = [
            "/hc"
                .GET(health_check),
            "/api/users"
                .POST(create_user)
                .PATCH(update_user),
        ];
    }
}
