//! # Handler Components
//! 
//! This module provides various components for typed, declarative way to
//! extract request data and construct response data:
//! 
//! - [`body`]: for handling request and response bodies
//! - [`header`]: for handling request headers
//! - [`param`]: for handling request parameters (path and query)
//! - [`status`]: for handling response HTTP status codes
//! 
//! See individual modules or each component's documentation for details.
//! 
//! ## Example
//! 
//! ```
//! use ohkami::component::{Path, Json, status};
//! use ohkami::serde::Serialize;
//! 
//! #[derive(Deserialize)]
//! struct CreateUserRequest<'req> {
//!     name: &'req str,
//! }
//! 
//! #[derive(Serialize)]
//! struct User {
//!     id: u64,
//!     name: String,
//! }
//! 
//! # enum AppError {}
//! # impl ohkami::IntoResponse for AppError {
//! #     fn into_response(self) -> ohkami::Response {
//! #         todo!()
//! #     }
//! # }
//! 
//! async fn get_user(
//!     // Extract a path parameter as `u64`
//!     Path(id): Path<u64>,
//!             // Serialize `User` into `application/json` response body
//! ) -> Result<Json<User>, AppError> {
//!     Ok(Json(User {
//!         id,
//!         name: todo!(),
//!     }))
//! }
//! 
//! async fn create_user(
//!     // Extract `application/json` request body
//!     Json(body): Json<CreateUserRequest<'_>>,
//!             // Serialize `User` into `application/json` response body
//!             // with `201 Created` status
//! ) -> Result<status::Created<Json<User>>, AppError> {
//!     Ok(status::Created(Json(User {
//!         id: todo!(),
//!         name: body.name.to_owned(),
//!     })))
//! }
//! ```

pub mod body;
pub mod header;
pub mod param;
pub mod status;

pub use body::Json;
pub use header::Cookie;
pub use param::{Path, Query};

#[cold] #[inline(never)]
fn reject(msg: impl std::fmt::Display) -> crate::Response {
    crate::Response::BadRequest().with_text(msg.to_string())
}

#[cfg(feature="openapi")]
mod bound {
    use crate::openapi;
    use serde::{Serialize, Deserialize};

    pub trait Schema: openapi::Schema {}
    impl<S: openapi::Schema> Schema for S {}

    pub trait Incoming<'req>: Deserialize<'req> + openapi::Schema {}
    impl<'req, T> Incoming<'req> for T where T: Deserialize<'req> + openapi::Schema {}

    pub trait Outgoing: Serialize + openapi::Schema {}
    impl<T> Outgoing for T where T: Serialize + openapi::Schema {}
}
#[cfg(not(feature="openapi"))]
mod bound {
    use serde::{Serialize, Deserialize};

    pub trait Schema: {}
    impl<S> Schema for S {}

    pub trait Incoming<'req>: Deserialize<'req> {}
    impl<'req, T> Incoming<'req> for T where T: Deserialize<'req> {}

    pub trait Outgoing: Serialize {}
    impl<T> Outgoing for T where T: Serialize {}
}
