#![allow(non_snake_case)]

use serde::Serialize;
use crate::{
    layer0_lib::{AsStr, Status, ContentType},
    layer1_req_res::{ResponseHeaders, Response, ErrResponse},
};


/// ## Response context
/// 
/// <br/>
/// 
/// ```ignore
/// async fn handler(c: Context) -> Response {
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
/// }
/// ```
/// 
/// <br/>
/// 
/// With error handling :
/// 
/// ```ignore
/// #[derive(Serialize)]
/// struct User {
///     id:       usize,
///     name:     String,
///     password: String,
/// }
/// 
/// #[Payload(JSON)]
/// struct CreateUser {
///     name:     String,
///     password: String,
/// }
/// 
/// async fn create_user(
///     c:    Context,
///     body: CreateUser,
/// ) -> Response<User> {
///     let created_id = insert_user_returing_id(
///         &body.name,
///         &body.password,
///     ).await /* Result<usize, MyError> */
///         .map_err(|e| c
///             .InternalError()      // generate a `ErrResponse`
///             .Text("in DB operation") // add message if needed
///         )?; // early return in error cases
/// 
///     c.Created(User {
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
    #[inline(always)] pub fn Text<Text: AsStr>(&self, text: Text) -> Response<Text> {
        Response::ok_with_body_asstr(
            text,
            Status::OK,
            ContentType::Text,
            &self.headers,
        )
    }
    #[inline(always)] pub fn HTML<HTML: AsStr>(&self, html: HTML) -> Response<HTML> {
        Response::ok_with_body_asstr(
            html,
            Status::OK,
            ContentType::HTML,
            &self.headers,
        )
    }
    #[inline(always)] pub fn JSON<JSON: Serialize>(&self, json: JSON) -> Response<JSON> {
        Response::ok_with_body_json(
            json,
            Status::OK,
            &self.headers,
        )
    }

    #[inline(always)] pub fn Created<JSON: Serialize>(&self, entity: JSON) -> Response<JSON> {
        Response::ok_with_body_json(
            entity,
            Status::Created,
            &self.headers,
        )
    }

    #[inline(always)] pub fn NoContent(&self) -> Response<()> {
        Response::ok_without_body(
            Status::NoContent,
            &self.headers,
        )
    }
}

impl Context {
    #[inline(always)] pub fn Redirect(&self, location: impl AsStr) -> Response {
        Response::redirect(
            location,
            Status::Found,
            &self.headers,
        )
    }
    #[inline(always)] pub fn RedirectPermanently(&self, location: impl AsStr) -> Response {
        Response::redirect(
            location,
            Status::MovedPermanently,
            &self.headers,
        )
    }
}

macro_rules! impl_error_response {
    ($( $name:ident ),*) => {
        impl Context {
            $(
                #[inline(always)] pub fn $name(&self) -> ErrResponse {
                    ErrResponse::new(Status::$name, &self.headers)
                }
            )*
        }
    };
} impl_error_response!(
    BadRequest,
    Unauthorized,
    Forbidden,
    // NotFound,            -- customizable by GlobalFangs, not pub ↓
    // InternalServerError, -- too long name ↓
    NotImplemented
); impl Context {
    #[inline(always)] pub fn InternalError(&self) -> ErrResponse {
        ErrResponse::new(Status::InternalServerError, &self.headers)
    }
} impl Context {
    #[inline(always)] pub(crate) fn NotFound(&self) -> ErrResponse {
        ErrResponse::new(
            Status::NotFound,
            &self.headers,
        )
    }
}




#[cfg(test)] mod __ {use crate::Context;
    #[test] fn test_context_change_header() {
        let mut c = Context::new();
        let __now__ = crate::layer0_lib::now();

        // newly set
        c.headers.Server("ohkami");
        assert_eq!(c.headers.to_string(), format!("\
            Connection: Keep-Alive\r\n\
            Keep-Alive: timout=5\r\n\
            Date: {__now__}\r\n\
            Server: ohkami\r\n\
        "));

        c.headers.ETag("identidentidentident");
        assert_eq!(c.headers.to_string(), format!("\
            Connection: Keep-Alive\r\n\
            Keep-Alive: timout=5\r\n\
            Date: {__now__}\r\n\
            Server: ohkami\r\n\
            ETag: identidentidentident\r\n\
        "));

        // remove
        c.headers.Server(None);
        assert_eq!(c.headers.to_string(), format!("\
            Connection: Keep-Alive\r\n\
            Keep-Alive: timout=5\r\n\
            Date: {__now__}\r\n\
            ETag: identidentidentident\r\n\
        "));

        // update
        c.headers.Server("ohkami2");
        c.headers.ETag("new-etag");
        assert_eq!(c.headers.to_string(), format!("\
            Connection: Keep-Alive\r\n\
            Keep-Alive: timout=5\r\n\
            Date: {__now__}\r\n\
            Server: ohkami2\r\n\
            ETag: new-etag\r\n\
        "));

        // custom
        c.headers.custom("X-MyApp-Cred", "abcdefg");
        c.headers.custom("MyApp-Data", "gfedcba");
        assert_eq!(c.headers.to_string(), format!("\
            Connection: Keep-Alive\r\n\
            Keep-Alive: timout=5\r\n\
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
        assert_eq!(c.Text("Hello, world!").to_string(), format!("\
            HTTP/1.1 200 OK\r\n\
            Content-Type: text/plain\r\n\
            Content-Length: 13\r\n\
            Connection: Keep-Alive\r\n\
            Keep-Alive: timout=5\r\n\
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
        assert_eq!(c.Created(User{ id:42, name:"kanarus", age:19 }).to_string(), format!("\
            HTTP/1.1 201 Created\r\n\
            Content-Type: application/json\r\n\
            Content-Length: 35\r\n\
            Connection: Keep-Alive\r\n\
            Keep-Alive: timout=5\r\n\
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
        assert_eq!(c.Created(serde_json::json!({"id":42,"name":"kanarus","age":19})).to_string(), format!("\
            HTTP/1.1 201 Created\r\n\
            Content-Type: application/json\r\n\
            Content-Length: 35\r\n\
            Connection: Keep-Alive\r\n\
            Keep-Alive: timout=5\r\n\
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
        assert_eq!(c.Created(r#"{"id":42,"name":"kanarus","age":19}"#).to_string(), format!("\
            HTTP/1.1 201 Created\r\n\
            Content-Type: application/json\r\n\
            Content-Length: 45\r\n\
            Connection: Keep-Alive\r\n\
            Keep-Alive: timout=5\r\n\
            Date: {__now__}\r\n\
            Server: ohkami\r\n\
            ETag: identidentidentident\r\n\
            \r\n\
        ") + r##""{\"id\":42,\"name\":\"kanarus\",\"age\":19}""##);

        c.headers.Server(None);
        assert_eq!(c.NoContent().to_string(), format!("\
            HTTP/1.1 204 No Content\r\n\
            Connection: Keep-Alive\r\n\
            Keep-Alive: timout=5\r\n\
            Date: {__now__}\r\n\
            ETag: identidentidentident\r\n\
            \r\n\
        "));

        c.headers.Server("ohkami2");
        c.headers.ETag("new-etag");
        assert_eq!(c.BadRequest().to_string(), format!("\
            HTTP/1.1 400 Bad Request\r\n\
            Connection: Keep-Alive\r\n\
            Keep-Alive: timout=5\r\n\
            Date: {__now__}\r\n\
            Server: ohkami2\r\n\
            ETag: new-etag\r\n\
            \r\n\
        "));

        c.headers.custom("X-MyApp-Cred", "abcdefg");
        c.headers.custom("MyApp-Data", "gfedcba");
        assert_eq!(c.InternalError().Text("I'm sorry fo").to_string(), format!("\
            HTTP/1.1 500 Internal Server Error\r\n\
            Connection: Keep-Alive\r\n\
            Keep-Alive: timout=5\r\n\
            Date: {__now__}\r\n\
            Server: ohkami2\r\n\
            ETag: new-etag\r\n\
            MyApp-Data: gfedcba\r\n\
            X-MyApp-Cred: abcdefg\r\n\
            Content-Type: text/plain\r\n\
            Content-Length: 12\r\n\
            \r\n\
            I'm sorry fo\
        "));
    }
}
