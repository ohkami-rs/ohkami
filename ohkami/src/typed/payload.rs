use crate::{FromRequest, IntoResponse, Response};
use serde::{Serialize, Deserialize};


pub trait Payload: Sized {
    type Type: PayloadType;
}

pub trait PayloadType {
    const MIME_TYPE: &'static str;
    const CONTENT_TYPE: &'static str = Self::MIME_TYPE;

    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl crate::serde::de::Error>;
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
