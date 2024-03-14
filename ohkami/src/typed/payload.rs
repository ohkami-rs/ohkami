use crate::{FromRequest, IntoResponse, Response};
use serde::{Serialize, Deserialize};


pub trait Payload: Sized {
    type Type: PayloadType;
}

pub trait PayloadType {
    const CONTENT_TYPE: &'static str;

    type Error: std::error::Error;

    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, Self::Error>;
    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, Self::Error>;
}

const _: () = {
    macro_rules! content_type {
        () => {<<Self as Payload>::Type as PayloadType>::CONTENT_TYPE};
    }

    impl<'req, P> FromRequest<'req> for P
    where
        P: Payload + Deserialize<'req>
    {
        type Error = Response;

        fn from_request(req: &'req crate::Request) -> Result<Self, Self::Error> {
            if req.headers.ContentType() != Some(content_type!()) {
                return Err(
                    (|| Response::BadRequest().text(format!("{} payload is required", content_type!())))()
                );
            }

            let Some(bytes) = req.payload() else {
                return Err(
                    (|| Response::BadRequest().text("Payload is required"))()
                );
            };

            <<Self as Payload>::Type as PayloadType>::parse(bytes)
                .map_err(|e| {
                    eprintln!("Failed to parse {} as {}: {e}", std::any::type_name::<Self>(), content_type!());
                    Response::BadRequest().text("Unexpected payload format")
                })
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
                macro_rules! content_type {
                    () => {<<Self as Payload>::Type as PayloadType>::CONTENT_TYPE};
                }

                let bytes = match <<Self as Payload>::Type as PayloadType>::bytes(&self) {
                    Ok(bytes) => bytes,
                    Err(e) => return (|| {
                        eprintln!("Failed to serialize {} as {}: {e}", std::any::type_name::<Self>(), content_type!());
                        Response::InternalServerError()
                    })()
                };

                res.headers.set()
                    .ContentType(content_type!())
                    .ContentLength(bytes.len().to_string());

                res.content = Some(std::borrow::Cow::Owned(bytes));
            }
            res
        }
    }
};
