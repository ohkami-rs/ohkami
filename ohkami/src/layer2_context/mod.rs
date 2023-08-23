#![allow(non_snake_case)]

use crate::{
    layer0_lib::{AsStr, Status},
    layer1_req_res::{ResponseHeaders, Response},
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
/// ```ignore
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
    pub headers: ResponseHeaders,
}

impl Context {
    #[inline(always)] pub(crate) fn new() -> Self {
        Self { headers: ResponseHeaders::new() }
    }
}

impl Context {
    #[inline] pub fn OK(&self) -> Response {
        Response {
            status:  Status::OK,
            headers: self.headers.to_string(),
            content: None,
        }
    }
    #[inline] pub fn Created(&self) -> Response {
        Response {
            status:  Status::Created,
            headers: self.headers.to_string(),
            content: None,
        }
    }
    #[inline] pub fn NoContent(&self) -> Response {
        Response {
            status:  Status::NoContent,
            headers: self.headers.to_string(),
            content: None,
        }
    }
}

macro_rules! impl_error_response {
    ($( $name:ident ),*) => {
        impl Context {
            $(
                #[inline] pub fn $name(&self) -> Response {
                    Response {
                        status:  Status::$name,
                        headers: self.headers.to_string(),
                        content: None,
                    }
                }
            )*
        }
    };
} impl_error_response!(
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    InternalServerError,
    NotImplemented
);

impl Context {
    #[inline] pub fn redirect_to(&self, location: impl AsStr) -> Response {
        let mut headers = self.headers.to_string();
        headers.push_str("Location: ");
        headers.push_str(location.as_str());
        headers.push('\r');
        headers.push('\n');

        Response {
            status:  Status::Found,
            content: None,
            headers,
        }
    }
    #[inline] pub fn redirect_permanently(&self, location: impl AsStr) -> Response {
        let mut headers = self.headers.to_string();
        headers.push_str("Location: ");
        headers.push_str(location.as_str());
        headers.push('\r');
        headers.push('\n');

        Response {
            status:  Status::MovedPermanently,
            content: None,
            headers,
        }
    }
}




#[cfg(test)] mod __ {use crate::Context;
    #[test] fn test_context_change_header() {
        let mut c = Context::new();
        let __now__ = crate::layer0_lib::now();

        // newly set
        c.headers.Server("ohkami");
        assert_eq!(c.headers.to_string(), format!("\
            Date: {__now__}\r\n\
            Server: ohkami\r\n\
        "));

        c.headers.ETag("identidentidentident");
        assert_eq!(c.headers.to_string(), format!("\
            Date: {__now__}\r\n\
            Server: ohkami\r\n\
            ETag: identidentidentident\r\n\
        "));

        // remove
        c.headers.Server(None);
        assert_eq!(c.headers.to_string(), format!("\
            Date: {__now__}\r\n\
            ETag: identidentidentident\r\n\
        "));

        // update
        c.headers.Server("ohkami2");
        c.headers.ETag("new-etag");
        assert_eq!(c.headers.to_string(), format!("\
            Date: {__now__}\r\n\
            Server: ohkami2\r\n\
            ETag: new-etag\r\n\
        "));

        // custom
        c.headers.custom("X-MyApp-Cred", "abcdefg");
        c.headers.custom("MyApp-Data", "gfedcba");
        assert_eq!(c.headers.to_string(), format!("\
            Date: {__now__}\r\n\
            Server: ohkami2\r\n\
            ETag: new-etag\r\n\
            MyApp-Data: gfedcba\r\n\
            X-MyApp-Cred: abcdefg\r\n\
        "));
    }

    #[test] fn test_context_generate_response() {
        let mut c = Context::new();
        let __now__ = crate::layer0_lib::now();

        c.headers.Server("ohkami");
        assert_eq!(std::str::from_utf8(&c.OK().text("Hello, world!").into_bytes()).unwrap(), format!("\
            HTTP/1.1 200 OK\r\n\
            Content-Type: text/plain; charset=utf-8\r\n\
            Content-Length: 13\r\n\
            Date: {__now__}\r\n\
            Server: ohkami\r\n\
            \r\n\
            Hello, world!\
        "));

        c.headers.ETag("identidentidentident");

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

        c.headers.Server(None);
        assert_eq!(std::str::from_utf8(&c.NoContent().into_bytes()).unwrap(), format!("\
            HTTP/1.1 204 No Content\r\n\
            Date: {__now__}\r\n\
            ETag: identidentidentident\r\n\
            \r\n\
        "));

        c.headers.Server("ohkami2");
        c.headers.ETag("new-etag");
        assert_eq!(std::str::from_utf8(&c.BadRequest().into_bytes()).unwrap(), format!("\
            HTTP/1.1 400 Bad Request\r\n\
            Date: {__now__}\r\n\
            Server: ohkami2\r\n\
            ETag: new-etag\r\n\
            \r\n\
        "));

        c.headers.custom("X-MyApp-Cred", "abcdefg");
        c.headers.custom("MyApp-Data", "gfedcba");
        assert_eq!(std::str::from_utf8(&c.InternalServerError().text("I'm sorry fo").into_bytes()).unwrap(), format!("\
            HTTP/1.1 500 Internal Server Error\r\n\
            Content-Type: text/plain; charset=utf-8\r\n\
            Content-Length: 12\r\n\
            Date: {__now__}\r\n\
            Server: ohkami2\r\n\
            ETag: new-etag\r\n\
            MyApp-Data: gfedcba\r\n\
            X-MyApp-Cred: abcdefg\r\n\
            \r\n\
            I'm sorry fo\
        "));
    }
}
