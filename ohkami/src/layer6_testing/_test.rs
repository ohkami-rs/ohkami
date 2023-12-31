use crate::__rt__;

use crate::prelude::*;
use crate::testing::*;
use crate::{Fang, IntoFang, http::Status};


#[__rt__::test] async fn testing_example_simple() {
    let simple_ohkami = Ohkami::new(());

    let res = simple_ohkami.oneshot(TestRequest::GET("/")).await;
    assert_eq!(res.status(), Status::NotFound);

    let hello_ohkami = Ohkami::new((
        "/hello".
            GET(hello),
    ));

    let res = hello_ohkami.oneshot(TestRequest::GET("/")).await;
    assert_eq!(res.status(), Status::NotFound);

    let res = hello_ohkami.oneshot(TestRequest::GET("/hello")).await;
    assert_eq!(res.status(), Status::OK);
    assert_eq!(res.text(), Some("Hello, world!"));
}

async fn hello(c: Context) -> Response {
    c.OK().text("Hello, world!")
}


#[__rt__::test] async fn testing_example_complex() {
    let users_ohkami = Ohkami::with((), (
        "/".   PUT(create_user),
        "/:id".GET(get_user),
    ));

    let testing_example = Ohkami::with((SetServerHeader,), (
        "/health".GET(health_check),
        "/users" .By(users_ohkami),
    ));

    let res = testing_example.oneshot(TestRequest::GET("/")).await;
    assert_eq!(res.status(), Status::NotFound);

    let res = testing_example.oneshot(TestRequest::GET("/health")).await;
    assert_eq!(res.status(), Status::NoContent);
    assert_eq!(res.header("Server").unwrap(), "ohkami");

    let res = testing_example.oneshot(TestRequest::GET("/users/100")).await;
    assert_eq!(res.status(), Status::NotFound);
    assert_eq!(res.header("Server").unwrap(),  "ohkami");

    let res = testing_example.oneshot(TestRequest::GET("/users/42")).await;
    assert_eq!(res.status(), Status::OK);
    assert_eq!(
        res.json::<serde_json::Value>().unwrap().unwrap(),
        serde_json::json!({"name":"kanarus","age":20}),
    );

    let res = testing_example.oneshot(TestRequest::PUT("/users")).await;
    assert_eq!(res.status(), Status::BadRequest);

    let res = testing_example.oneshot(TestRequest::PUT("/users")
        .json(CreateUser {
            name: "kanarus",
            age:  None,
        })).await;
    assert_eq!(res.status(), Status::Created);
    assert_eq!(res.header("Server").unwrap(),   "ohkami");
    assert_eq!(
        res.json::<serde_json::Value>().unwrap().unwrap(),
        serde_json::json!({"name":"kanarus","age":0}),
    );
}

struct SetServerHeader;
impl IntoFang for SetServerHeader {
    fn into_fang(self) -> Fang {
        Fang(|c: &mut Context| {
            c.set_headers()
                .Server("ohkami");
        })
    }
}

async fn health_check(c: Context) -> Response {
    c.NoContent()
}

#[derive(serde::Serialize)]
struct User {
    name: String,
    age:  u8,
}

async fn get_user(c: Context, id: usize) -> Response {
    match id {
        42 => c.OK().json(User {
            name: format!("kanarus"),
            age:  20,
        }),
        _ => c.NotFound()
    }
}

#[derive(serde::Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
struct CreateUser<'c> {
    name: &'c str,
    age:  Option<u8>,
}
// Can't use `#[Payload(JSON)]` here becasue this test is within `ohkami`
impl<'req> crate::FromRequest<'req> for CreateUser<'req> {
    type Error = ::std::borrow::Cow<'static, str>;
    fn parse(req: &'req Request) -> Result<Self, ::std::borrow::Cow<'static, str>> {
        let Some(payload) = req.payload()
            else {return Err(::std::borrow::Cow::Borrowed("Expected a payload"))};
        match req.headers.ContentType() {
            Some("application/json") => serde_json::from_slice(payload)
                .map_err(|_| ::std::borrow::Cow::Owned(format!("Failed to deserialize payload"))),
            _ => Err(::std::borrow::Cow::Borrowed("Expected a json payload"))
        }
    }
}
async fn create_user<'h>(c: Context, payload: CreateUser<'h>) -> Response {
    c.Created().json(User {
        name: payload.name.to_string(),
        age:  payload.age.unwrap_or(0),
    })
}
