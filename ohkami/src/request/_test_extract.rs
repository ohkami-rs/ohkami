#![cfg(debug_assertions)]
#![cfg(all(feature="rt_tokio", feature="DEBUG"))]

use crate::prelude::*;
use crate::testing::*;
use crate::{typed::status, format::{JSON, Query}};
use ::serde::Deserialize;

#[cfg(feature="openapi")]
use crate::openapi;


#[derive(Deserialize)]
#[allow(unused)]
struct User<'req> {
    name:     &'req str,
    password: &'req str,
}
#[cfg(feature="openapi")]
impl<'req> openapi::support::Schema for User<'req> {
    const NAME: &'static str = "User";
    fn schema() -> impl Into<openapi::schema::SchemaRef> {
        openapi::Schema::object()
            .property("name", openapi::Schema::string())
            .property("password", openapi::Schema::string())
    }
}

#[derive(Deserialize)]
struct HelloQuery<'req> {
    name:   &'req str,
    repeat: Option<usize>,
}
#[cfg(feature="openapi")]
impl<'req> openapi::support::Schema for HelloQuery<'req> {
    const NAME: &'static str = "HelloQuery";
    fn schema() -> impl Into<openapi::schema::SchemaRef> {
        openapi::Schema::object()
            .property("name", openapi::Schema::string())
            .optional("repeat", openapi::Schema::integer())
    }
}

#[crate::__rt__::test] async fn extract_required_payload() {
    async fn create_user(
        JSON(_user): JSON<User<'_>>
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
        body: Option<JSON<User<'_>>>,
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
        query: Option<Query<HelloQuery<'_>>>,
    ) -> String {
        match query {
            None => String::from("no query"),
            Some(Query(HelloQuery { name, repeat })) =>
                format!("Hello, {name}!").repeat(repeat.unwrap_or(1))
        }
    }

    let t = Ohkami::new((
        "/".GET(hello),
    )).test();

    {
        let req = TestRequest::GET("/")
            .query("name", "ohkami");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("Hello, ohkami!"));
    }

    {
        let req = TestRequest::GET("/");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("no query"));
    }

    {
        let req = TestRequest::GET("/")
            .query("name",   "ohkami")
            .query("repeat", "3");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some(
            "Hello, ohkami!Hello, ohkami!Hello, ohkami!"
        ));
    }
}
