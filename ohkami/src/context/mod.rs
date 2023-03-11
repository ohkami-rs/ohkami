pub(crate) mod store;

use async_std::{sync::{Arc, Mutex, MutexGuard}};
use serde::Serialize;
use self::store::Store;
use crate::{
    response::{
        Response, components::{
            header::ResponseHeaders,
            body::{Text, Html, Json},
            status::Status,
            content_type::ContentType,
        }
    },
};


pub struct Context {
    pub(crate) cache: Arc<Mutex<Store>>,
    pub(crate) additional_headers: ResponseHeaders,
}

impl Context {
    #[inline] pub(crate) fn new(cache: Arc<Mutex<Store>>) -> Self {
        Self {
            cache,
            additional_headers: ResponseHeaders::new(),
        }
    }

    #[inline] pub async fn cache(&self) -> MutexGuard<Store> {
        self.cache.lock().await
    }
    #[inline] pub fn set_header(&mut self, key: &'static str, value: &'static str) {
        self.additional_headers.set(key, value)
    }
}

#[allow(non_snake_case)]
impl Context {
    #[inline] pub fn text<T: Text>(&self, body: T) -> Response<T> {
        Response::with_body(
            Status::OK,
            ContentType::text_plain,
            &self.additional_headers,
            body.as_str()
        )
    }
    #[inline] pub fn html<H: Html>(&self, body: H) -> Response<H> {
        Response::with_body(
            Status::OK,
            ContentType::text_html,
            &self.additional_headers,
            body.as_str()
        )
    }
    #[inline] pub fn json<'j, J: Json<'j>>(&self, body: J) -> Response<J> {
        Response::with_body(
            Status::OK,
            ContentType::application_json,
            &self.additional_headers,
            &body.as_str()
        )
    }
    #[inline] pub fn Created<B: Serialize>(&self, body: B) -> Response<B> {
        Response::with_body(
            Status::Created,
            ContentType::application_json,
            &self.additional_headers,
            &serde_json::to_string(&body).unwrap()
        )
    }
    #[inline] pub fn NoContent(&self) -> Response<()> {
        Response::no_content(&self.additional_headers)
    }
}

macro_rules! impl_error_responses {
    {$( $name:ident ),*} => {
        $(
            impl Context {
                #[allow(non_snake_case)]
                #[inline] pub fn $name<T: Serialize, Msg: Text>(&self, message: Msg) -> Response<T> {
                    Response::error(
                        Status::$name,
                        &self.additional_headers,
                        message
                    )
                }
            }
        )*
    };
} impl_error_responses! {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    InternalServerError,
    NotImplemented
}
