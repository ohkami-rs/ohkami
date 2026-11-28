//! # Ohkami's built-in middlewares
//!
//! Each of them is a [`Fang`](crate::Fang), which can be applied both as
//! a **global fang** to an `Ohkami` or as a **local fang** to a specific handler.
//!
//! - [`BasicAuth`], [`Cors`], [`Csrf`], [`Jwt`] (authentication/security)
//! - [`Context`] (request context)
//! - [`Enamel`] (security headers; experimental)
//! - `Timeout` (handling timeout; native runtimes only)
//!
//! To implement your own middleware, see [`FangAction`](crate::FangAction)
//! for simple cases and [`Fang`](crate::Fang) for full control.

mod basicauth;
pub use basicauth::BasicAuth;

mod cors;
pub use cors::Cors;

mod csrf;
pub use csrf::Csrf;

mod jwt;
pub use jwt::{Jwt, JwtToken};

mod context;
pub use context::Context;

pub mod enamel;
pub use enamel::Enamel;

#[cfg(feature = "__rt_native__")]
mod timeout;
#[cfg(feature = "__rt_native__")]
pub use timeout::Timeout;
