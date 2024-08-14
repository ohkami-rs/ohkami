//! Format of a part of request/response
//! 
//! ## Builtin
//! 
//! - `Query` - query parameters
//! - `JSON` - payload of application/json
//! - `Multipart` - payload of multipart/form-data
//! - `URLEncoded` - payload of application/x-www-form-urlencoded
//! - `Text` - payload of text/plain
//! - `HTML` - payload of text/html
//! 
//! ## Note
//! 
//! Every format should:
//! 
//! 1. have the structure of `struct {Name}<Schema>(pub Schema);`
//! 2. properly handle result of `ohkami::format::validated`

mod builtin;
pub use builtin::*;


#[cfg(feature="nightly")]
/// ## In-place validation for schema struct in a format
/// 
/// Format holding a type implementing this
/// will perform in-place validation when
/// 
/// - parsing request body to the type
/// - building response body of the type
/// 
/// <br>
/// 
/// ### required
/// 
/// - `type ErrorMessage: Display` (directly used for an error response text)
/// - `fn validate`
pub trait V {
    type ErrorMessage: std::fmt::Display;
    fn validate(&self) -> Result<(), Self::ErrorMessage>;
}

#[doc(hidden)]
#[inline]
pub fn validated<S>(schema: S) -> Result<S, crate::Response> {
    #[allow(non_camel_case_types)]
    trait schema: Sized {
        fn validated(self) -> Result<Self, crate::Response>;
    }
    impl<T> schema for T {
        #[cfg(not(feature="nightly"))]
        #[inline(always)]
        fn validated(self) -> Result<Self, crate::Response> {Ok(self)}

        #[cfg(feature="nightly")]
        #[inline(always)]
        default fn validated(self) -> Result<Self, crate::Response> {Ok(self)}
    }
    #[cfg(feature="nightly")]
    impl<S: V> schema for S {
        #[inline]
        fn validated(self) -> Result<Self, crate::Response> {
            self.validate().map_err(|e| crate::Response::BadRequest().with_text(e.to_string()))?;
            Ok(self)
        }
    }

    schema.validated()
}
