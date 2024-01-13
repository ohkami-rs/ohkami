#![cfg(feature="utils")]

use crate::__rt__;
use crate::prelude::*;
use crate::testing::*;
use crate::utils::{Text, ResponseBody};


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

async fn hello() -> impl IntoResponse {
    Text("Hello, world!")
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
    assert_eq!(res.status(), Status::NotImplemented);
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
        Fang(|res: &mut Response| {
            res.headers.set()
                .Server("ohkami");
        })
    }
}

enum APIError {
    TODO,
}
impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        Status::NotImplemented.into_response()
    }
}


async fn health_check() -> impl IntoResponse {
    Status::NoContent
}

#[derive(serde::Serialize)]
struct User {
    name: String,
    age:  u8,
}
impl ResponseBody for User {
    fn into_response_with(self, status: Status) -> Response {
        Response::with(status).json(self)
    }
}

async fn get_user(id: usize) -> Result<OK<User>, APIError> {
    match id {
        42 => Ok(OK(User {
            name: format!("kanarus"),
            age:  20,
        })),
        _ => Err(APIError::TODO)
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
    type Error = crate::FromRequestError;
    fn from_request(req: &'req Request) -> Result<Self, crate::FromRequestError> {
        let Some(payload) = req.payload()
            else {return Err(crate::FromRequestError::Static("Expected a payload"))};
        match req.headers.ContentType() {
            Some("application/json") => serde_json::from_slice(payload)
                .map_err(|_| crate::FromRequestError::Owned(format!("Failed to deserialize payload"))),
            _ => Err(crate::FromRequestError::Static("Expected a json payload"))
        }
    }
}
async fn create_user(payload: CreateUser<'_>) -> Created<User> {
    Created(User {
        name: payload.name.to_string(),
        age:  payload.age.unwrap_or(0),
    })
}
