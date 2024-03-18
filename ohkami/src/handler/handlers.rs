#![allow(non_snake_case)]

use crate::Ohkami;
use super::{Handler, IntoHandler};
use crate::ohkami::router::RouteSections;


pub struct Handlers {
    pub(crate) route:   RouteSections,
    pub(crate) GET:     Option<Handler>,
    pub(crate) PUT:     Option<Handler>,
    pub(crate) POST:    Option<Handler>,
    pub(crate) PATCH:   Option<Handler>,
    pub(crate) DELETE:  Option<Handler>,
} impl Handlers {
    fn new(route_str: &'static str) -> Self {
        Self {
            route:   RouteSections::from_literal(route_str),
            GET:     None,
            PUT:     None,
            POST:    None,
            PATCH:   None,
            DELETE:  None,
        }
    }
}

macro_rules! Handlers {
    ($( $method:ident ),*) => {
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
    pub(crate) route: RouteSections,
    pub(crate) ohkami: Ohkami,
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
        }
    };
} Route! { GET, PUT, POST, PATCH, DELETE }




#[cfg(feature="utils")]
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
    } impl<'req> FromRequest<'req> for CreateUser<'req> {
        type Error = crate::FromRequestError;
        fn from_request(req: &'req crate::Request) -> Result<Self, Self::Error> {
            let payload = req.payload().ok_or_else(|| crate::FromRequestError::Static("Payload expected"))?;
            match req.headers.ContentType() {
                Some("application/json") => serde_json::from_slice(payload).map_err(|e| crate::FromRequestError::Owned(e.to_string())),
                _ => Err(crate::FromRequestError::Static("Payload expected")),
            }
        }
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
    } impl<'req> FromRequest<'req> for UpdateUser<'req> {
        type Error = crate::FromRequestError;
        fn from_request(req: &'req crate::Request) -> Result<Self, Self::Error> {
            let payload = req.payload().ok_or_else(|| Self::Error::Static("Payload expected"))?;
            match req.headers.ContentType() {
                Some("application/json") => serde_json::from_slice(payload).map_err(|e| Self::Error::Owned(e.to_string())),
                _ => Err(Self::Error::Static("Payload expected")),
            }
        }
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
