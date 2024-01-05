#![allow(non_snake_case)]

use crate::{layer3_fang_handler::RouteSections, layer5_ohkami::Ohkami};
use super::{Handler, IntoHandler};


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
                pub fn $method<Args>(mut self, handler: impl IntoHandler<Args>) -> Self {
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
        pub trait Route {
            $(
                fn $method<Args>(self, handler: impl IntoHandler<Args>) -> Handlers;
            )*
            fn By(self, another: Ohkami) -> ByAnother;
        }
        impl Route for &'static str {
            $(
                fn $method<Args>(self, handler: impl IntoHandler<Args>) -> Handlers {
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




#[cfg(test)] #[allow(unused)] mod __ {
    use std::borrow::Cow;
    use serde::{Serialize, Deserialize};
    use super::{Handlers, Route};
    use crate::{response as r, FromRequest, Responder, Response};


    enum APIError {
        DBError,
    }
    impl Responder for APIError {
        fn respond_to(self, _: &crate::Request) -> crate::Response {
            Response::InternalServerError()
        }
    }

    async fn health_check() -> r::Empty {
        r::Empty::NoContent()
    }

    #[derive(Serialize)]
    struct User {
        id:       usize,
        name:     String,
        password: String,
    }

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
        type Error = Cow<'static, str>;
        fn parse(req: &'req crate::Request) -> Result<Self, ::std::borrow::Cow<'static, str>> {
            let payload = req.payload().ok_or_else(|| Cow::Borrowed("Payload expected"))?;
            match req.headers.ContentType() {
                Some("application/json") => serde_json::from_slice(payload).map_err(|e| Cow::Owned(e.to_string())),
                _ => Err(Cow::Borrowed("Payload expected")),
            }
        }
    }

    async fn create_user<'req>(payload: CreateUser<'req>) -> Result<r::JSON<User>, APIError> {
        let CreateUser { name, password } = payload;

        mock::authenticate().await?;

        let id = mock::DB.insert_returning_id(CreateUser{ name, password }).await?;

        Ok(r::JSON::Created(User {
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
        type Error = Cow<'static, str>;
        fn parse(req: &'req crate::Request) -> Result<Self, ::std::borrow::Cow<'static, str>> {
            let payload = req.payload().ok_or_else(|| Cow::Borrowed("Payload expected"))?;
            match req.headers.ContentType() {
                Some("application/json") => serde_json::from_slice(payload).map_err(|e| Cow::Owned(e.to_string())),
                _ => Err(Cow::Borrowed("Payload expected")),
            }
        }
    }

    async fn update_user<'req>(body: UpdateUser<'req>) -> Result<r::Empty, APIError> {
        mock::authenticate().await?;
        mock::DB.update_returning_id(body).await?;

        Ok(r::Empty::NoContent())
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
