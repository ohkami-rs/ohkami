#![allow(non_snake_case)]
#![cfg(feature="testing")]

use crate::__rt__;
use crate::prelude::*;
use crate::testing::*;


fn my_ohkami() -> Ohkami {
    let health_ohkami = Ohkami::new((
        "/".GET(|| async {"health_check"}),
    ));

    let profiles_ohkami = Ohkami::new((
        "/:username"
            .GET(|username: String| async  move {
                format!("get_profile of user `{username}`")
            }),
        "/:username/follow"
            .POST(|username: String| async move {
                format!("follow_user `{username}`")
            })
            .DELETE(|username: String| async move {
                format!("unfollow_user `{username}`")
            })
    ));

    let articles_ohkami = Ohkami::new((
        "/"
            .GET(|| async {"get_article"})
            .POST(|| async {"post_article"}),
        "/feed"
            .GET(|| async {"get_feed"}),
        "/:slug".By(Ohkami::new((
            "/"
                .GET(|slug: String| async move {
                    format!("get_article {slug}")
                })
                .PUT(|slug: String| async move {
                    format!("put_article {slug}")
                })
                .DELETE(|slug: String| async move {
                    format!("delete_article {slug}")
                }),
            "/comments"
                .POST(|slug: String| async move {
                    format!("post_comments {slug}")
                })
                .GET(|slug: String| async move {
                    format!("get_comments {slug}")
                }),
            "/comments/:id"
                .DELETE(|(slug, id): (String, usize)| async move {
                    format!("delete_comment {slug} / {id}")
                }),
            "/favorite"
                .POST(|slug: String| async move {
                    format!("favorite_article {slug}")
                })
                .DELETE(|slug: String| async move {
                    format!("unfavorite_article {slug}")
                }),
        )))
    ));

    Ohkami::new((
        "/health".By(health_ohkami),
        "/api".By(Ohkami::new((
            "/profiles".By(profiles_ohkami),
            "/articles".By(articles_ohkami),
        ))),
    ))
}

#[crate::__rt__::test] async fn test_handler_registration() {
    let t = my_ohkami();


    /* GET /health */

    let req = TestRequest::GET("/health");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("health_check"));


    /* GET /api/profiles/:username */

    let req = TestRequest::GET("/api/profiles");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::NotFound);

    let req = TestRequest::GET("/api/profiles/123");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("get_profile of user `123`"));


    /* POST,DELETE /api/profiles/:username/follow */

    let req = TestRequest::GET("/api/profiles/the_user/follow");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::NotFound);

    let req = TestRequest::POST("/api/profiles/the_user");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::NotFound);

    let req = TestRequest::POST("/api/profiles/the_user/follow");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::OK);

    let req = TestRequest::POST("/api/profiles/the_user/follow");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("follow_user `the_user`"));

    let req = TestRequest::DELETE("/api/profiles/the_user/follow");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("unfollow_user `the_user`"));

    /* GET /api/articles/feed */

    let req = TestRequest::GET("/api/articles/feed");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("get_feed"));


    /* GET,PUT,DELETE /api/articles/:slug */

    let req = TestRequest::GET("/api/articles/ohkami123456");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("get_article ohkami123456"));

    let req = TestRequest::PUT("/api/articles/abcdef123");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("put_article abcdef123"));


    /* DELETE /api/articles/:slug/comments/:id */

    let req = TestRequest::DELETE("/api/articles/__prototype__/comments/42");
    let res = t.oneshot(req).await;
    assert_eq!(res.text(), Some("delete_comment __prototype__ / 42"));
}


#[crate::__rt__::test] async fn test_fang_registration() {
    use std::sync::{OnceLock, Mutex};
    use crate::{Fang, FangProc};

    fn N() -> &'static Mutex<usize> {
        static N: OnceLock<Mutex<usize>> = OnceLock::new();
        N.get_or_init(|| Mutex::new(0))
    }

    struct Increment;
    impl<Inner: FangProc> Fang<Inner> for Increment {
        type Proc = IncrementProc<Inner>;
        fn chain(&self, inner: Inner) -> Self::Proc {
            IncrementProc(inner)
        }
    }
    struct IncrementProc<Inner: FangProc>(Inner);
    impl<Inner: FangProc> FangProc for IncrementProc<Inner> {
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send + 'b {
            #[cfg(feature="DEBUG")]
            println!("Called `Increment`");

            *N().lock().unwrap() += 1;
            self.0.bite(req)
        }
    }

    async fn h() -> &'static str {"h"}


    /*===== with no nests =====*/
    *N().lock().unwrap() = 0;

    let o = Ohkami::with((Increment,), (
        "/a"  .GET(h),
        "/a/b".GET(h),
    ));

    dbg!(o.clone().into_router());
    dbg!(o.clone().into_router().into_radix());

    let req = TestRequest::GET("/a");
    o.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 1);

    let req = TestRequest::GET("/a");
    o.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 2);

    let req = TestRequest::GET("/a/b");
    o.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 3);

    
    /*===== with nests =====*/
    *N().lock().unwrap() = 0;

    let o = Ohkami::with((Increment,), (
        "/a"  .GET(h),
        "/a/b".GET(h),
        "/a/b/c".By(Ohkami::with((), (
            "/d"  .GET(h),
            "/d/e".GET(h),
        )))
    ));

    dbg!(o.clone().into_router());
    dbg!(o.clone().into_router().into_radix());

    let req = TestRequest::GET("/a");
    o.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 1);

    let req = TestRequest::GET("/a/b");
    o.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 2);
    let req = TestRequest::GET("/a/b/c/d");
    o.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 3);

    let req = TestRequest::GET("/a/b/c/d/e");
    o.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 4);
}

#[__rt__::test] async fn test_fangs_nesting() {
    use std::sync::{Mutex, OnceLock};
    use crate::{Fang, FangProc};

    #[allow(non_snake_case)]
    fn MESSAGES() -> &'static Mutex<String> {
        static MESSAGES: OnceLock<Mutex<String>> = OnceLock::new();
        MESSAGES.get_or_init(|| Mutex::new(String::new()))
    }

    struct HelloFang {
        name: &'static str
    }
    impl<I: FangProc> Fang<I> for HelloFang {
        type Proc = HelloFangProc<I>;
        fn chain(&self, inner: I) -> Self::Proc {
            HelloFangProc { inner, name: self.name }
        }
    }
    struct HelloFangProc<I: FangProc> {
        name:  &'static str,
        inner: I,
    }
    impl<I: FangProc> FangProc for HelloFangProc<I> {
        async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
            MESSAGES().lock().unwrap().push_str(self.name);
            self.inner.bite(req).await
        }
    }

}

#[__rt__::test] async fn test_pararell_registering() {
    async fn hello_help() -> &'static str {
        "Hi, this is `hello` api. \
        Call me with your name as a path parameter:\n\
        \t `GET /hello/{you name here}`"
    }

    async fn hello(name: std::borrow::Cow<'_, str>) -> String {
        format!("Hello, {name}!")
    }


    /* register static pattern in ahead */

    let t = Ohkami::new((
        "/hello/help" .GET(hello_help),
        "/hello/:name".GET(hello),
    ));

    let req = TestRequest::GET("/hello/help");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::OK);
    assert_eq!(res.text(),   Some(
        "Hi, this is `hello` api. \
        Call me with your name as a path parameter:\n\
        \t `GET /hello/{you name here}`"
    ));

    let req = TestRequest::GET("/hello/Mr.%20wolf");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::OK);
    assert_eq!(res.text(),   Some("Hello, Mr. wolf!"));


    /* register param pattern in ahead */

    let t = Ohkami::new((
        "/hello/:name".GET(hello),
        "/hello/help" .GET(hello_help),
    ));

    let req = TestRequest::GET("/hello/help");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::OK);
    assert_eq!(res.text(),   Some(
        "Hi, this is `hello` api. \
        Call me with your name as a path parameter:\n\
        \t `GET /hello/{you name here}`"
    ));

    let req = TestRequest::GET("/hello/Mr.%20wolf");
    let res = t.oneshot(req).await;
    assert_eq!(res.status(), Status::OK);
    assert_eq!(res.text(),   Some("Hello, Mr. wolf!"));
}
