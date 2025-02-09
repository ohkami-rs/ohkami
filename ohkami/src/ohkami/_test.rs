#![allow(non_snake_case)]
#![cfg(all(test, feature="__rt_native__", feature="DEBUG"))]

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

#[test] fn test_handler_registration() {
    let t = my_ohkami().test();

    crate::__rt__::testing::block_on(async {

        /* GET /health */

        let req = TestRequest::GET("/health");
        let get_res = t.oneshot(req).await;
        assert_eq!(get_res.text(), Some("health_check"));

        let req = TestRequest::HEAD("/health");
        let head_res = t.oneshot(req).await;
        assert_eq!(head_res.text(), None);
        assert_eq!(
            {let mut h = get_res.headers().collect::<Vec<_>>(); h.sort(); h},
            {let mut h = head_res.headers().collect::<Vec<_>>(); h.sort(); h}
        );

        /* GET /api/profiles/:username */

        let req = TestRequest::GET("/api/profiles");
        let get_res = t.oneshot(req).await;
        assert_eq!(get_res.status(), Status::NotFound);

        let req = TestRequest::HEAD("/api/profiles");
        let head_res = t.oneshot(req).await;
        assert_eq!(head_res.text(), None);
        assert_eq!(
            {let mut h = get_res.headers().collect::<Vec<_>>(); h.sort(); h},
            {let mut h = head_res.headers().collect::<Vec<_>>(); h.sort(); h}
        );

        let req = TestRequest::GET("/api/profiles/123");
        let get_res = t.oneshot(req).await;
        assert_eq!(get_res.text(), Some("get_profile of user `123`"));

        let req = TestRequest::HEAD("/api/profiles/123");
        let head_res = t.oneshot(req).await;
        assert_eq!(head_res.text(), None);
        assert_eq!(
            {let mut h = get_res.headers().collect::<Vec<_>>(); h.sort(); h},
            {let mut h = head_res.headers().collect::<Vec<_>>(); h.sort(); h}
        );


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

    });
}


#[test] fn test_fang_registration() {
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
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send {
            #[cfg(feature="DEBUG")]
            println!("Called `Increment`");

            *N().lock().unwrap() += 1;
            self.0.bite(req)
        }
    }

    async fn h() -> &'static str {"h"}

    crate::__rt__::testing::block_on(async {

        /*===== with no nests =====*/
        *N().lock().unwrap() = 0;

        let t = Ohkami::new((Increment,
            "/a"  .GET(h),
            "/a/b".GET(h),
        )).test();

        let req = TestRequest::GET("/a");
        t.oneshot(req).await;
        assert_eq!(*N().lock().unwrap(), 1);

        let req = TestRequest::GET("/a");
        t.oneshot(req).await;
        assert_eq!(*N().lock().unwrap(), 2);

        let req = TestRequest::GET("/a/b");
        t.oneshot(req).await;
        assert_eq!(*N().lock().unwrap(), 3);


        /*===== with nests =====*/
        *N().lock().unwrap() = 0;

        let t = Ohkami::new((Increment,
            "/a"  .GET(h),
            "/a/b".GET(h),
            "/a/b/c".By(Ohkami::new((
                "/d"  .GET(h),
                "/d/e".GET(h),
            )))
        )).test();

        let req = TestRequest::GET("/a");
        t.oneshot(req).await;
        assert_eq!(*N().lock().unwrap(), 1);

        let req = TestRequest::GET("/a/b");
        t.oneshot(req).await;
        assert_eq!(*N().lock().unwrap(), 2);
        let req = TestRequest::GET("/a/b/c/d");
        t.oneshot(req).await;
        assert_eq!(*N().lock().unwrap(), 3);

        let req = TestRequest::GET("/a/b/c/d/e");
        t.oneshot(req).await;
        assert_eq!(*N().lock().unwrap(), 4);

    });
}

#[test] fn test_fangs_nesting() {
    use std::sync::{Mutex, OnceLock};
    use crate::{Fang, FangProc, Ohkami};

    #[allow(non_snake_case)]
    fn MESSAGES() -> &'static Mutex<Vec<String>> {
        static MESSAGES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
        MESSAGES.get_or_init(|| Mutex::new(Vec::new()))
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
            {
                let mut lock = MESSAGES().lock().unwrap();
                lock.push(format!("Hello, {}!", self.name));
            }
            let res = self.inner.bite(req).await;
            {
                let mut lock = MESSAGES().lock().unwrap();
                lock.push(format!("Bye, {}!", self.name));
            }
            res
        }
    }

    async fn h() -> &'static str {"handler"}


    let t = Ohkami::new((
        HelloFang { name: "Amelia" },
        "/abc".GET(h),
        "/def".By(Ohkami::new((
            HelloFang { name: "Brooks" },
            HelloFang { name: "Carter" },
            "/".GET(h),
            "/jkl".By(Ohkami::new((
                HelloFang { name: "Daniel" },
                "/mno".GET(h),
            )))
        ))),
        "/pqr".By(Ohkami::new((
            HelloFang { name: "Evelyn" },
            "/stu".GET(h),
        ))),
    )).test();

    crate::__rt__::testing::block_on(async {
        {MESSAGES().lock().unwrap().clear();
            let req = TestRequest::GET("/abc");
            let res = t.oneshot(req).await;

            assert_eq!(res.status(), Status::OK);
            assert_eq!(&*MESSAGES().lock().unwrap(), &[
                "Hello, Amelia!",
                "Bye, Amelia!",
            ]);
        }

        {MESSAGES().lock().unwrap().clear();
            let req = TestRequest::GET("/def");
            let res = t.oneshot(req).await;

            assert_eq!(res.status(), Status::OK);
            assert_eq!(&*MESSAGES().lock().unwrap(), &[
                "Hello, Amelia!",
                "Hello, Brooks!",
                "Hello, Carter!",
                "Bye, Carter!",
                "Bye, Brooks!",
                "Bye, Amelia!",
            ]);
        }

        {MESSAGES().lock().unwrap().clear();
            let req = TestRequest::GET("/def/jklmno");
            let res = t.oneshot(req).await;

            assert_eq!(res.status(), Status::NotFound);
            assert_eq!(&*MESSAGES().lock().unwrap(), &[
                "Hello, Amelia!",
                "Hello, Brooks!",
                "Hello, Carter!",
                "Bye, Carter!",
                "Bye, Brooks!",
                "Bye, Amelia!",
            ]);
        }

        {MESSAGES().lock().unwrap().clear();
            let req = TestRequest::GET("/def/jkl/mno");
            let res = t.oneshot(req).await;

            assert_eq!(res.status(), Status::OK);
            assert_eq!(&*MESSAGES().lock().unwrap(), &[
                "Hello, Amelia!",
                "Hello, Brooks!",
                "Hello, Carter!",
                "Hello, Daniel!",
                "Bye, Daniel!",
                "Bye, Carter!",
                "Bye, Brooks!",
                "Bye, Amelia!",
            ]);
        }
    });
}

#[test] fn test_pararell_registering() {
    async fn hello_help() -> &'static str {
        "Hi, this is `hello` api. \
        Call me with your name as a path parameter:\n\
        \t `GET /hello/{you name here}`"
    }

    async fn hello(name: std::borrow::Cow<'_, str>) -> String {
        format!("Hello, {name}!")
    }

    crate::__rt__::testing::block_on(async {

        /* register static pattern in ahead */

        let t = Ohkami::new((
            "/hello/help" .GET(hello_help),
            "/hello/:name".GET(hello),
        )).test();

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
        )).test();

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

    });
}

#[test]
fn duplcate_routes_registration() {
    let _ = Ohkami::new((
        "/abc".GET(|| async {"GET"}),
        "/abc".PUT(|| async {"PUT"}),
    ));
}

#[test]
fn with_global_fangs() {
    async fn list_pets() -> &'static str {"list"}
    async fn create_pet() -> &'static str {"created"}
    async fn show_pet_by_id() -> &'static str {"found"}

    use std::sync::{Mutex, LazyLock};

    fn count() -> &'static Mutex<usize> {
        static COUNT: LazyLock<Mutex<usize>> =
            LazyLock::new(|| Mutex::new(0));
        &*COUNT
    }

    #[derive(Clone)]
    struct Logger;
    impl FangAction for Logger {
        async fn fore<'a>(&'a self, _req: &'a mut Request) -> Result<(), Response> {
            *count().lock().unwrap() += 1;
            Ok(())
        }
    }

    /* global fangs */
    crate::__rt__::testing::block_on(async {
        let t = Ohkami::new((Logger,
            "/pets"
                .GET(list_pets)
                .POST(create_pet),
            "/pets/:petId"
                .GET(show_pet_by_id),
        )).test();
        {
            let req = TestRequest::GET("/");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
            assert_eq!(res.text(), None);
            assert_eq!(*count().lock().unwrap(), 1); // called even when NotFound
        }
        {
            let req = TestRequest::GET("/pets");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), Some("list"));
            assert_eq!(*count().lock().unwrap(), 2);
        }
    });

    /* local fangs */
    crate::__rt__::testing::block_on(async {
        let t = Ohkami::new((
            "/pets"
                .GET((Logger, list_pets))
                .POST((Logger, create_pet)),
            "/pets/:petId"
                .GET((Logger, show_pet_by_id)),
        )).test();
        {
            let req = TestRequest::GET("/");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
            assert_eq!(res.text(), None);
            assert_eq!(*count().lock().unwrap(), 2); // not changed from previous one
        }
        {
            let req = TestRequest::GET("/pets");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), Some("list"));
            assert_eq!(*count().lock().unwrap(), 3);
        }
    });
}

#[test]
fn method_dependent_fang_applying() {
    {
        #[derive(Clone)]
        struct SomeFang;
        impl crate::prelude::FangAction for SomeFang {}

        async fn handler() {}

        let _ = Ohkami::new((
            "/users"
                .GET(handler)
                .POST(handler),
            "/users/:id"
                .GET(handler),
            "/users/:id".By(Ohkami::new((SomeFang, "/"
                .PUT(handler),
            ))),
            "/tweets"
                .GET(handler),
            "/tweets".By(Ohkami::new((SomeFang, "/"
                .POST(handler),
            )))
        )); // no panic
    }

    crate::__rt__::testing::block_on(async {
        use std::sync::{Mutex, LazyLock};

        fn global_count() -> &'static Mutex<usize> {
            static GLOBAL_COUNT: LazyLock<Mutex<usize>> =
                LazyLock::new(|| Mutex::new(0));
            &*GLOBAL_COUNT
        }

        fn local_count() -> &'static Mutex<usize> {
            static LOCAL_COUNT: LazyLock<Mutex<usize>> =
                LazyLock::new(|| Mutex::new(0));
            &*LOCAL_COUNT
        }

        #[derive(Clone)]
        struct Logger;
        impl FangAction for Logger {
            async fn fore<'a>(&'a self, _req: &'a mut Request) -> Result<(), Response> {
                *global_count().lock().unwrap() += 1;
                Ok(())
            }
        }

        #[derive(Clone)]
        struct Auth;
        impl FangAction for Auth {
            async fn fore<'a>(&'a self, _req: &'a mut Request) -> Result<(), Response> {
                *local_count().lock().unwrap() += 1;
                Ok(())
            }
        }
        
        #[derive(Clone)]
        struct Count2;
        impl FangAction for Count2 {
            async fn fore<'a>(&'a self, _req: &'a mut Request) -> Result<(), Response> {
                *local_count().lock().unwrap() += 2;
                Ok(())
            }
        }
        
        let t = Ohkami::new((Logger, // applies `Logger` on any route
            "/"
                .GET(|| async {"Hello, GET"}),
            "/".By(Ohkami::new((
                // locally applies `Auth` for `PUT /`
                "/"
                    .PUT((Auth, || async {"Hello, PUT"})),
            ))),
            "/auth".By(Ohkami::new((
                "/"
                    .GET(|| async {"auth page"}),
            ))),
            "/auth".By(Ohkami::new((Count2, // applies `Count2` on any `/auth`
                "/"
                    .PUT(|| async {"authed"}),
            ))),
            "/auth".By(Ohkami::new((
                // locally applies `Auth` for `POST /auth`, `DELETE /auth/d`
                "/"
                    .POST((Auth, || async {"auth control"})),
                "/d"
                    .DELETE((Auth, || async {"deleted"})),
            )))
        )).test();

        {
            let req = TestRequest::GET("/");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), Some("Hello, GET"));
            assert_eq!(*global_count().lock().unwrap(), 1);
            assert_eq!(*local_count().lock().unwrap(), 0);
        }
        {
            // Logger (with_global) + Auth (with)
            let req = TestRequest::PUT("/");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), Some("Hello, PUT"));
            assert_eq!(*global_count().lock().unwrap(), 2);
            assert_eq!(*local_count().lock().unwrap(), 1);
        }
        {
            // Logger (with_global) + Count2 (with_global)
            let req = TestRequest::GET("/auth");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), Some("auth page"));
            assert_eq!(*global_count().lock().unwrap(), 3);
            assert_eq!(*local_count().lock().unwrap(), 3);
        }
        {
            // Logger (with_global) + Count2 (with_global)
            let req = TestRequest::PUT("/auth");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), Some("authed"));
            assert_eq!(*global_count().lock().unwrap(), 4);
            assert_eq!(*local_count().lock().unwrap(), 5);
        }
        {
            // Logger (with_global) + Auth (with) + Count2 (with_global)
            let req = TestRequest::POST("/auth");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), Some("auth control"));
            assert_eq!(*global_count().lock().unwrap(), 5);
            assert_eq!(*local_count().lock().unwrap(), 8);
        }
        {
            // Logger (with_global) + Auth (with) + Count2 (with_global)
            let req = TestRequest::DELETE("/auth/d");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), Some("deleted"));
            assert_eq!(*global_count().lock().unwrap(), 6);
            assert_eq!(*local_count().lock().unwrap(), 11);
        }

        {
            // Logger (with_global)
            let req = TestRequest::GET("/wrong");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
            assert_eq!(res.text(), None);
            assert_eq!(*global_count().lock().unwrap(), 7);
            assert_eq!(*local_count().lock().unwrap(), 11);
        }
        {
            // Logger (with_global) + Count2 (with_global)
            let req = TestRequest::GET("/auth/wrong");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
            assert_eq!(res.text(), None);
            assert_eq!(*global_count().lock().unwrap(), 8);
            assert_eq!(*local_count().lock().unwrap(), 13);
        }
        {
            // Logger (with_global) + Count2 (with_global)
            let req = TestRequest::DELETE("/auth/wrong");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
            assert_eq!(res.text(), None);
            assert_eq!(*global_count().lock().unwrap(), 9);
            assert_eq!(*local_count().lock().unwrap(), 15); // <--
        }
    });
}

#[test] fn prefixy_routes() {
    crate::__rt__::testing::block_on(async {
        let t = Ohkami::new((
            "/abcd".GET(|| async {"This is abcd"}),
            "/abc".GET(|| async {"This is abc"}),
        )).test();

        {
            let req = TestRequest::GET("/abc");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), Some("This is abc"));
        } {
            let req = TestRequest::GET("/ab");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
        } {
            let req = TestRequest::GET("/abc2");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
        } {
            let req = TestRequest::GET("/abcd");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), Some("This is abcd"));
        } {
            let req = TestRequest::GET("/abcde");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
        }

        /* reversed; MUST have the same behavior */

        let t = Ohkami::new((
            "/abc".GET(|| async {"This is abc"}),
            "/abcd".GET(|| async {"This is abcd"}),
        )).test(); {
            let req = TestRequest::GET("/abc");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), Some("This is abc"));
        } {
            let req = TestRequest::GET("/ab");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
        } {
            let req = TestRequest::GET("/abc2");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
        } {
            let req = TestRequest::GET("/abcd");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), Some("This is abcd"));
        } {
            let req = TestRequest::GET("/abcde");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
        }
    });
}

#[test]
#[should_panic =
    "handler `ohkami::ohkami::_test::panics_unexpected_path_params::hello_name` \
    requires 1 path param(s) \
    BUT the route `/hello` captures only 0 param(s)"
]
fn panics_unexpected_path_params() {
    async fn hello_name(name: &str) -> String {
        format!("Hello, {name}!")
    }

    let _ = Ohkami::new((
        "/hello".GET(hello_name),
    )).test(); /* panics here on finalize */
}

#[test]
#[should_panic =
    "handler `ohkami::ohkami::_test::check_path_params_counted_accumulatedly::hello_name_age` \
    requires 2 path param(s) \
    BUT the route `/hello/:name` captures only 1 param(s)"
]
fn check_path_params_counted_accumulatedly() {
    async fn hello_name(name: &str) -> String {
        format!("Hello, {name}!")
    }
    async fn hello_name_age((name, age): (&str, u8)) -> String {
        format!("Hello, {name} ({age})!")
    }

    let _ = Ohkami::new((
        "/hello/:name".By(Ohkami::new((
            "/".GET(hello_name),
        ))),
    )).test(); /* NOT panics here */

    let _ = Ohkami::new((
        "/hello/:name".By(Ohkami::new((
            "/".GET(hello_name_age),
        ))),
    )).test(); /* panics here */
}
