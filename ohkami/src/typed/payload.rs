use crate::{FromRequest, IntoResponse, Response};
use serde::{Serialize, Deserialize};


/// # Request/Response Payload
/// 
/// <br>
/// 
/// - `T` can be used as request body if `T: Payload + Deserialize`
/// - `T` can be used as response body if `T: Payload + Serialize`
/// 
/// in ohkami's *typed* system.
/// 
/// It's recommended to impl this by `#[Payload]` attribute with a `PayloadType` argument.
/// 
/// <br>
/// 
/// ---
/// *base.rs*
/// ```
/// /* This trait and the attribute */
/// use ohkami::typed::Payload;
/// 
/// /* A `PayloadType` for application/json payload */
/// use ohkami::builtin::payload::JSON;
/// 
/// use ohkami::serde::{Deserialize, Serialize};
/// 
/// #[Payload]
/// #[derive(
///     Deserialize, // derive to use `User` as request body
///     Serialize    // derive to use `User` as response body
/// )]
/// struct User {
///     id:   usize,
///     name: String,
/// }
/// ```
/// ---
/// 
/// <br>
/// 
/// We derive `Deserialize` or `Serialize` in most cases, so `#[Payload]` supports shorthands for that:
/// 
/// <br>
/// 
/// ---
/// *shorthand.rs*
/// ```
/// use ohkami::typed::Payload;
/// use ohkami::builtin::payload::JSON;
/// 
/// #[Payload(JSON/DS)]
/// struct User {
///     id:   usize,
///     name: String,
/// }
/// ```
/// ---
/// 
/// <br>
/// 
/// After `/`,
/// 
/// - `D` automatically derives `Desrerialize` for the struct
/// - `S` automatically derives `Serialize` for the struct
/// 
/// respectively.
pub trait Payload: Sized {
    type Type: PayloadType;
}

pub trait PayloadType {
    /// Mime type like `application/json`, `text/plain`, ...
    /// 
    /// Used for checking `Content-Type` of a request.
    const MIME_TYPE: &'static str;
    
    /// Just mime type, or maybe it with some additional information, like `application/json`, `text/plain; charset=UTF-8`, ...
    /// 
    /// Used for `Content-Type` of a response with the `Payload` type.
    const CONTENT_TYPE: &'static str = Self::MIME_TYPE;

    /// Deserializing logic for parsing a request body that should be the `Payload` type.
    /// 
    /// <br>
    /// 
    /// ---
    /// *example.rs*
    /// ```
    /// # use serde::Deserialize;
    /// fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl crate::serde::de::Error> {
    ///     ::serde_json::from_slice(bytes)
    /// }
    /// ```
    /// ---
    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl crate::serde::de::Error>;

    /// Serializing logic for a response body of the `Payload` type.
    /// 
    /// <br>
    /// 
    /// ---
    /// *example.rs*
    /// ```
    /// # use serde::Serialize;
    /// fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, impl crate::serde::ser::Error> {
    ///     ::serde_json::to_vec(&value)
    /// }
    /// ```
    /// ---
    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, impl crate::serde::ser::Error>;
}

const _: () = {
    impl<'req, P> FromRequest<'req> for P
    where
        P: Payload + Deserialize<'req> + 'req
    {
        type Error = Response;

        #[inline(always)]
        fn from_request(req: &'req crate::Request) -> Result<Self, Self::Error> {
            req.payload()
                .ok_or_else(|| Response::BadRequest().text(
                    format!("{} payload is required",
                        <<Self as Payload>::Type as PayloadType>::MIME_TYPE
                    )
                ))?
                .map_err(|e| Response::BadRequest().text(e.to_string()))
        }
    }

    impl<P> IntoResponse for P
    where
        P: Payload + Serialize
    {
        #[inline]
        fn into_response(self) -> Response {
            let mut res = Response::OK();
            {
                let content_type = <<Self as Payload>::Type as PayloadType>::CONTENT_TYPE;

                let bytes = match <<Self as Payload>::Type as PayloadType>::bytes(&self) {
                    Ok(bytes) => bytes,
                    Err(e) => return (|| {
                        eprintln!("Failed to serialize {} as {}: {e}", std::any::type_name::<Self>(), content_type);
                        Response::InternalServerError()
                    })()
                };

                res.headers.set()
                    .ContentType(content_type)
                    .ContentLength(bytes.len().to_string());

                res.content = Some(std::borrow::Cow::Owned(bytes));
            }
            res
        }
    }
};
