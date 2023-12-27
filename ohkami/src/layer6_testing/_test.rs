use crate::__rt__;

use crate::prelude::*;
use crate::testing::*;
use crate::{Fang, IntoFang, http::Status};


#[__rt__::test] async fn testing_example_simple() {
    let simple_ohkami = Ohkami::new(());

    let res = simple_ohkami.oneshot(TestRequest::GET("/")).await;
    assert_eq!(res.status, Status::NotFound);

    let hello_ohkami = Ohkami::new((
        "/hello".
            GET(hello),
    ));

    let res = hello_ohkami.oneshot(TestRequest::GET("/")).await;
    assert_eq!(res.status, Status::NotFound);

    let res = hello_ohkami.oneshot(TestRequest::GET("/hello")).await;
    assert_eq!(res.status, Status::OK);
    assert_eq!(res.content.unwrap().text(), Some("Hello, world!"));
}

async fn hello(c: Context) -> Response {
    c.OK().text("Hello, world!")
}


#[__rt__::test] async fn testing_example_complex() {
    let users_ohkami = Ohkami::with((SetCustomHeaders,), (
        "/".   PUT(create_user),
        "/:id".GET(get_user),
    ));

    let testing_example = Ohkami::with((SetServerHeader,), (
        "/health".GET(health_check),
        "/users" .By(users_ohkami),
    ));

    let res = testing_example.oneshot(TestRequest::GET("/")).await;
    assert_eq!(res.status, Status::NotFound);

    let res = testing_example.oneshot(TestRequest::GET("/health")).await;
    assert_eq!(res.status, Status::NoContent);
    assert_eq!(res.headers.get("Server").unwrap(), "ohkami");
    assert_eq!(res.headers.get("X-State"), None);

    let res = testing_example.oneshot(TestRequest::GET("/users/100")).await;
    assert_eq!(res.status, Status::NotFound);
    assert_eq!(res.headers.get("Server").unwrap(),  "ohkami");
    assert_eq!(res.headers.get("X-State").unwrap(), "testing");

    let res = testing_example.oneshot(TestRequest::GET("/users/42")).await;
    assert_eq!(res.status, Status::OK);
    assert_eq!(res.content.unwrap().json().unwrap(), r#"{"name":"kanarus","age":20}"#);

    let res = testing_example.oneshot(TestRequest::PUT("/users")).await;
    assert_eq!(res.status, Status::BadRequest);

    let res = testing_example.oneshot(TestRequest::PUT("/users")
        .json(CreateUser {
            name: format!("kanarus"),
            age:  None,
        })).await;
    assert_eq!(res.status, Status::Created);
    assert_eq!(res.headers.get("Server").unwrap(),   "ohkami");
    assert_eq!(res.headers.get("X-State").unwrap(),  "testing");
    assert_eq!(res.content.unwrap().json().unwrap(), r#"{"name":"kanarus","age":0}"#);
}

struct SetCustomHeaders;
impl IntoFang for SetCustomHeaders {
    fn bite(self) -> Fang {
        Fang(|c: &mut Context| {
            c.headers
                .custom("X-State", "testing");
        })
    }
}

struct SetServerHeader;
impl IntoFang for SetServerHeader {
    fn bite(self) -> Fang {
        Fang(|c: &mut Context| {
            c.headers
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
struct CreateUser {
    name: String,
    age:  Option<u8>,
}
// Can't use `#[Payload(JSON)]` here becasue this test is within `ohkami`
impl crate::FromRequest for CreateUser {
    type Error = ::std::borrow::Cow<'static, str>;
    fn parse(req: &Request) -> Result<Self, std::borrow::Cow<'static, str>> {
        let Some(("application/json", content)) = req.payload()
            else {return Err(std::borrow::Cow::Borrowed("Expected a json payload"))};
        serde_json::from_slice(content)
            .map_err(|_| std::borrow::Cow::Owned(format!("Failed to deserialize payload")))
    }
}
async fn create_user(c: Context, payload: CreateUser) -> Response {
    c.Created().json(User {
        name: payload.name,
        age:  payload.age.unwrap_or(0),
    })
}
