use crate::{FromRequest, IntoResponse, Request, Response};
use crate::response::Content;
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
/// #[Payload(JSON)]
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

    #[inline]
    fn extract<'req>(req: &'req Request) -> Option<Result<Self, impl crate::serde::de::Error>>
    where Self: Deserialize<'req> {
        let bytes = req.payload.as_ref()?;

        Some(if req.headers.ContentType().is_some_and(|ct|
            ct.starts_with(<Self::Type>::MIME_TYPE)
        ) {
            match <Self::Type>::parse::<Self>(unsafe {bytes.as_bytes()}) {
                Ok(this) => 'validation: {
                    if let Err(e) = this.validate() {
                        break 'validation Err((|| crate::serde::de::Error::custom(e.to_string()))())
                    }
                    Ok(this)
                }
                Err(err) => Err(err)
            }
        } else {
            #[cfg(debug_assertions)] {
                crate::warning!("Expected `{}` payload but found {}",
                    <Self::Type>::MIME_TYPE,
                    req.headers.ContentType().map(|ct| format!("`{ct}`")).unwrap_or(String::from("nothing"))
                )
            }
            
            Err((|| crate::serde::de::Error::custom(format!(
                "{} payload is required", <Self::Type>::MIME_TYPE
            )))())
        })
    }

    #[inline]
    fn inject(&self, res: &mut Response) -> Result<(), impl crate::serde::ser::Error>
    where Self: Serialize {
        self.validate().map_err(|e| crate::serde::ser::Error::custom(e.to_string()))?;
        match <Self::Type>::bytes(self) {
            Err(err)  => Err(err),
            Ok(bytes) => Ok({
                res.headers.set()
                    .ContentType(<Self::Type>::CONTENT_TYPE)
                    .ContentLength(bytes.len().to_string());
                res.content = Content::Payload(bytes.into());
            }),
        }
    }

    #[inline(always)]
    fn validate(&self) -> Result<(), impl std::fmt::Display> {
        Result::<(), std::convert::Infallible>::Ok(())
    }
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
    /// fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl ohkami::serde::de::Error> {
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
    /// fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, impl ohkami::serde::ser::Error> {
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
        fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
            Self::extract(req).map(|result| result.map_err(|e| {
                Response::BadRequest().with_text(e.to_string())
            }))
        }
    }

    impl<P> IntoResponse for P
    where
        P: Payload + Serialize
    {
        #[inline]
        fn into_response(self) -> Response {
            let mut res = Response::OK();
            if let Err(e) = self.inject(&mut res) {
                return (|| {
                    crate::warning!("Failed to serialize {} payload: {e}", <<Self as Payload>::Type>::MIME_TYPE);
                    Response::InternalServerError()
                })()
            }
            res
        }
    }
};
