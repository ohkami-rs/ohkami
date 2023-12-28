#![allow(non_snake_case)]

use crate::{
    layer0_lib::{Status, server_header},
    layer1_req_res::{Response},
};


/// ## Response context
/// 
/// <br/>
/// 
/// ```
/// use ohkami::prelude::*;
/// 
/// async fn handler(mut c: Context) -> Response {
///     // set header values
///     c.headers
///         .Server("ohkami")
///         .custom("X-MyApp-Cred", "abcdefg");
/// 
///     // update / delete header values
///     c.headers
///         .Server(None)
///         .custom("X-MyApp-Cred", "gfedcba");
/// 
///     // generate a `Response`
///     c.NoContent()
/// 
///     // `Content-Type`, `Content-Length`, `Date`,
///     // `Access-Control-*` are managed by ohkami.
/// }
/// ```
/// 
/// <br/>
/// 
/// With error handling :
/// 
/// ```
/// use ohkami::prelude::*;
/// use ohkami::utils::Payload;
/// 
/// #[derive(serde::Serialize)]
/// struct User {
///     id:       usize,
///     name:     String,
///     password: String,
/// }
/// 
/// #[Payload(JSON)]
/// #[derive(serde::Serialize)]
/// struct CreateUser {
///     name:     String,
///     password: String,
/// }
/// 
/// async fn create_user(
///     c:    Context,
///     body: CreateUser,
/// ) -> Response {
///     let Ok(created_id) = insert_user_returing_id(
///         &body.name,
///         &body.password,
///     ).await else {
///         return c.InternalServerError().text("in DB handling")
///     };
/// 
///     c.Created().json(User {
///         id:       created_id,
///         name:     body.name,
///         password: body.password,
///     })
/// }
/// ```
pub struct Context {
    #[cfg(feature="websocket")]
    pub(crate) upgrade_id: Option<crate::x_websocket::UpgradeID>,

    pub headers: server_header::Headers,
}

impl Context {
    #[inline(always)] pub fn set_headers(&mut self) -> server_header::SetHeaders<'_> {
        self.headers.set()
    }
}
impl Context {
    #[inline] pub(crate) fn new() -> Self {
        Self {
            #[cfg(feature="websocket")]
            upgrade_id: None,

            headers: server_header::Headers::new(),
        }
    }
}

macro_rules! generate_response {
    ($( $status:ident ),* $(,)?) => {$(
        impl Context {
            #[inline] pub fn $status(&self) -> Response {
                Response {
                    status:  Status::$status,
                    headers: self.headers.clone(),
                    content: None,
                }
            }
        }
    )*};
} generate_response! {
    SwitchingProtocols,

    OK,
    Created,
    NoContent,

    MovedPermanently,
    Found,

    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,

    InternalServerError,
    NotImplemented,
}

impl Context {
    #[inline] pub fn redirect_to(&self, location: impl Into<std::borrow::Cow<'static, str>>) -> Response {
        let mut headers = self.headers.clone();
        headers.set()
            .Location(location.into());

        Response {
            status:  Status::Found,
            content: None,
            headers,
        }
    }
    #[inline] pub fn redirect_permanently(&self, location: impl Into<std::borrow::Cow<'static, str>>) -> Response {
        let mut headers = self.headers.clone();
        headers.set()
            .Location(location.into());

        Response {
            status:  Status::MovedPermanently,
            content: None,
            headers,
        }
    }
}




#[cfg(test)] mod __ {use crate::Context;
    #[test] fn test_context_change_header() {
        use crate::layer0_lib::{
            now,
            server_header::{Header, Headers}
        };

        let mut c = Context::new();

        // newly set
        c.set_headers().Server("ohkami");
        assert_eq!(&c.headers, &Headers::from_iter([
            (Header::Date, now()),
            (Header::Server, "ohkami".to_string()),
        ]));

        c.set_headers().ETag("identidentidentident");
        assert_eq!(&c.headers,  &Headers::from_iter([
            (Header::Date, now()),
            (Header::Server, "ohkami".to_string()),
            (Header::ETag, "identidentidentident".to_string()),
        ]));

        // remove
        c.set_headers().Server(None);
        assert_eq!(&c.headers, &Headers::from_iter([
            (Header::Date, now()),
            (Header::ETag, "identidentidentident".to_string()),
        ]));

        // update
        c.set_headers().Server("ohkami2");
        c.set_headers().ETag("new-etag");
        assert_eq!(&c.headers, &Headers::from_iter([
            (Header::Date, now()),
            (Header::Server, "ohkami2".to_string()),
            (Header::ETag, "new-etag".to_string()),
        ]));
    }

    #[test] fn test_context_generate_response() {
        let mut c = Context::new();
        let __now__ = crate::layer0_lib::now();

        c.set_headers().Server("ohkami");
        assert_eq!(std::str::from_utf8(&c.OK().text("Hello, world!").into_bytes()).unwrap(), format!("\
            HTTP/1.1 200 OK\r\n\
            Content-Type: text/plain; charset=utf-8\r\n\
            Content-Length: 13\r\n\
            Date: {__now__}\r\n\
            Server: ohkami\r\n\
            \r\n\
            Hello, world!\
        "));

        c.set_headers().ETag("identidentidentident");

        // Checking how json serializing works in
        // structs and String...

        #[derive(serde::Serialize)] struct User {
            id:   usize,
            name: &'static str,
            age:  u8,
        }
        assert_eq!(std::str::from_utf8(&c.Created().json(User{ id:42, name:"kanarus", age:19 }).into_bytes()).unwrap(), format!("\
            HTTP/1.1 201 Created\r\n\
            Content-Type: application/json; charset=utf-8\r\n\
            Content-Length: 35\r\n\
            Date: {__now__}\r\n\
            Server: ohkami\r\n\
            ETag: identidentidentident\r\n\
            \r\n\
            {{\"id\":42,\"name\":\"kanarus\",\"age\":19}}\
        "));

        /* 
            `serde_json::Value::Object` uses `BTreeMap` for keys.
            So keys
                "id", "name", "age"
            are sorted to
                "age", "id", "name"
            in response body.
        */
        assert_eq!(std::str::from_utf8(&c.Created().json(serde_json::json!({"id":42,"name":"kanarus","age":19})).into_bytes()).unwrap(), format!("\
            HTTP/1.1 201 Created\r\n\
            Content-Type: application/json; charset=utf-8\r\n\
            Content-Length: 35\r\n\
            Date: {__now__}\r\n\
            Server: ohkami\r\n\
            ETag: identidentidentident\r\n\
            \r\n\
            {{\"age\":19,\"id\":42,\"name\":\"kanarus\"}}\
        "));

        /*
            This string "
                {"id":42,"name":"kanarus","age":19}
            " is interpreted as a **string** type json value r#`
                "{\"id\":42,\"name\":\"kanarus\",\"age\":19}"
            `#, **not an object** r#`
                {"id":42,"name":"kanarus","age":19}
            `#.
        */
        assert_eq!(std::str::from_utf8(&c.Created().json(r#"{"id":42,"name":"kanarus","age":19}"#).into_bytes()).unwrap(), format!("\
            HTTP/1.1 201 Created\r\n\
            Content-Type: application/json; charset=utf-8\r\n\
            Content-Length: 45\r\n\
            Date: {__now__}\r\n\
            Server: ohkami\r\n\
            ETag: identidentidentident\r\n\
            \r\n\
        ") + r##""{\"id\":42,\"name\":\"kanarus\",\"age\":19}""##);

        c.set_headers().Server(None);
        assert_eq!(std::str::from_utf8(&c.NoContent().into_bytes()).unwrap(), format!("\
            HTTP/1.1 204 No Content\r\n\
            Date: {__now__}\r\n\
            ETag: identidentidentident\r\n\
            \r\n\
        "));

        c.set_headers().Server("ohkami2");
        c.set_headers().ETag("new-etag");
        assert_eq!(std::str::from_utf8(&c.BadRequest().into_bytes()).unwrap(), format!("\
            HTTP/1.1 400 Bad Request\r\n\
            Date: {__now__}\r\n\
            Server: ohkami2\r\n\
            ETag: new-etag\r\n\
            \r\n\
        "));
    }
}
