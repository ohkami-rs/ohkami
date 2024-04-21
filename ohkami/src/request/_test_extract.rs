#![cfg(any(feature="rt_tokio", feature="rt_async-std"))]

use crate::prelude::*;
use crate::testing::*;
use crate::builtin::payload::JSON;
use crate::typed::{status, Payload};
use ::serde::Deserialize;


#[derive(Deserialize)]
#[allow(unused)]
struct User<'req> {
    name:     &'req str,
    password: &'req str,
} impl Payload for User<'_> {
    type Type = JSON;
}

#[derive(Deserialize)]
struct HelloQuery<'req> {
    name:   &'req str,
    repeat: Option<usize>,
} const _: () = {
    impl<'req> crate::FromRequest<'req> for HelloQuery<'req> {
        type Error = Response;
        fn from_request(req: &'req Request) -> Result<Self, Self::Error> {
            req.query().map_err(|_| Response::BadRequest())
        }
    }
    impl<'req> crate::FromRequest<'req> for Option<HelloQuery<'req>> {
        type Error = Response;
        fn from_request(req: &'req Request) -> Result<Self, Self::Error> {
            req.query().map_err(|_| Response::BadRequest())
        }
    }
};

#[crate::__rt__::test] async fn extract_required_payload() {
    async fn create_user(
        _body: User<'_>,
    ) -> status::Created {
        status::Created(())
    }

    let t = Ohkami::new((
        "/".POST(create_user),
    )).test();

    {
        let req = TestRequest::POST("/")
            .json_lit(r#"{
                "name": "kanarus",
                "password": "passw0rd"
            }"#);
        let _ = t.oneshot(req).await;
    }
}

#[crate::__rt__::test] async fn extract_optional_payload() {
    async fn post_user(
        body: Option<User<'_>>,
    ) -> &'static str {
        if body.is_none() {"none"} else {"some"}
    }

    let t = Ohkami::new((
        "/".POST(post_user),
    )).test();

    {
        let req = TestRequest::POST("/")
            .json_lit(r#"{
                "name": "kanarus",
                "password": "passw0rd"
            }"#);
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("some"));
    }

    {
        let req = TestRequest::POST("/");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("none"));
    }
}

#[crate::__rt__::test] async fn extract_optional_query() {
    async fn hello(
        query: Option<HelloQuery<'_>>,
    ) -> String {
        match query {
            None => String::from("no query"),
            Some(HelloQuery { name, repeat }) =>
                format!("Hello, {name}!").repeat(repeat.unwrap_or(1))
        }
    }

    let t = Ohkami::new((
        "/".POST(hello),
    )).test();

    {
        let req = TestRequest::POST("/")
            .query("name", "ohkami");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("Hello, ohkami!"));
    }

    {
        let req = TestRequest::POST("/");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("no query"));
    }

    {
        let req = TestRequest::POST("/")
            .query("name",   "ohkami")
            .query("repeat", "3");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some(
            "Hello, ohkami!Hello, ohkami!Hello, ohkami!"
        ));
    }

}
