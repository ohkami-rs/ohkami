#![allow(non_snake_case)]

use crate::layer3_fang_handler::RouteSections;
use super::{Handler, IntoHandler};


pub struct Handlers { route: RouteSections,
    GET:     Option<Handler>,
    PUT:     Option<Handler>,
    POST:    Option<Handler>,
    HEAD:    Option<Handler>,
    PATCH:   Option<Handler>,
    DELETE:  Option<Handler>,
    OPTIONS: Option<Handler>,
} impl Handlers {
    fn new(route_str: &'static str) -> Self {
        Self { route: RouteSections::from_literal(route_str),
            GET:     None,
            PUT:     None,
            POST:    None,
            HEAD:    None,
            PATCH:   None,
            DELETE:  None,
            OPTIONS: None,
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
} Handlers! { GET, PUT, POST, HEAD, PATCH, DELETE, OPTIONS }

macro_rules! Route {
    ($( $method:ident ),*) => {
        pub trait Route {
            $(
                fn $method<Args>(self, handler: impl IntoHandler<Args>) -> Handlers;
            )*
        }
        impl Route for &'static str {
            $(
                fn $method<Args>(self, handler: impl IntoHandler<Args>) -> Handlers {
                    let mut handlers = Handlers::new(self);
                    handlers.$method.replace(handler.into_handler());
                    handlers
                }
            )*
        }
    };
} Route! { GET, PUT, POST, HEAD, PATCH, DELETE, OPTIONS }




#[cfg(test)] #[allow(unused)] mod __ {
    use std::borrow::Cow;
    use serde::{Serialize, Deserialize};
    use super::{Handlers, Route};
    use crate::{
        Error,
        Context,
        Response,
        layer0_lib::ContentType,
        layer1_req_res::FromRequest,
    };

    async fn health_check(c: Context) -> Response<()> {
        c.NoContent()
    }

    #[derive(Serialize)]
    struct User {
        id:       usize,
        name:     String,
        password: String,
    }

    mod mock {
        pub async fn authenticate() -> Result<(), std::io::Error> {
            Ok(())
        }

        pub const DB: __::Database = __::Database; mod __ {
            pub struct Database;
            impl Database {
                pub async fn insert_returning_id(&self, Model: impl for<'de>serde::Deserialize<'de>) -> Result<usize, std::io::Error> {
                    Ok(42)
                }
                pub async fn update_returning_id(&self, Model: impl for<'de>serde::Deserialize<'de>) -> Result<usize, std::io::Error> {
                    Ok(24)
                }
            }
        }
    }

    // #[Payload(JSON)] : todo()!
    #[derive(Deserialize)]
    struct CreateUser {
        name:     String,
        password: String,
    } impl FromRequest for CreateUser {
        fn parse(req: &crate::Request) -> Result<Self, crate::Error> {
            let (content_type, body) = req.payload().ok_or_else(|| Error::Parse(Cow::Borrowed("Payload expected")))?;
            match content_type {
                ContentType::JSON => (),
                _ => return Err(Error::Parse(Cow::Borrowed("Payload expected"))),
            }

            // reexport json parsing function : todo!()
            serde_json::from_str(body)
                .map_err(|e| Error::Parse(Cow::Owned(e.to_string())))
        }
    }

    async fn create_user(c: Context, payload: CreateUser) -> Response<User> {
        let CreateUser { name, password } = payload;

        mock::authenticate().await
            .map_err(|e| c.Unauthorized())?;

        let id = mock::DB.insert_returning_id(CreateUser{
            name: name.clone(),
            password: password.clone(),
        }).await.map_err(|e| c.InternalError())?;

        c.Created(User { id, name, password })
    }

    // #[Payload(JSON)] : todo()!
    #[derive(Deserialize)]
    struct UpdateUser {
        name:     Option<String>,
        password: Option<String>,
    } impl FromRequest for UpdateUser {
        fn parse(req: &crate::Request) -> Result<Self, Error> {
            let (content_type, body) = req.payload().ok_or_else(|| Error::Parse(Cow::Borrowed("Payload expected")))?;
            match content_type {
                ContentType::JSON => (),
                _ => return Err(Error::Parse(Cow::Borrowed("Payload expected"))),
            }

            // reexport json parsing function : todo!()
            serde_json::from_str(body)
                .map_err(|e| Error::Parse(Cow::Owned(e.to_string())))
        }
    }

    async fn update_user(c: Context, req: UpdateUser) -> Response<()> {
        let UpdateUser { name, password } = req;

        mock::authenticate().await
            .map_err(|e| c.Unauthorized())?;

        mock::DB.update_returning_id(UpdateUser {
            name: name.clone(),
            password: password.clone(),
        }).await.map_err(|e| c.InternalError())?;

        c.NoContent()
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
