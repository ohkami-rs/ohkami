#![cfg(all(feature="testing", feature="utils"))]
use crate::prelude::*;
use crate::testing::*;
use crate::utils::Text;


fn my_ohkami() -> Ohkami {
    let health_ohkami = Ohkami::new((
        "/".GET(|| async {Text("health_check")}),
    ));

    let profiles_ohkami = Ohkami::new((
        "/:username"
            .GET(|username: String| async  move {
                Text(format!("get_profile of user `{username}`"))
            }),
        "/:username/follow"
            .POST(|username: String| async move {
                Text(format!("follow_user `{username}`"))
            })
            .DELETE(|username: String| async move {
                Text(format!("unfollow_user `{username}`"))
            })
    ));

    let articles_ohkami = Ohkami::new((
        "/"
            .GET(|| async {Text("get_article")})
            .POST(|| async {Text("post_article")}),
        "/feed"
            .GET(|| async {Text("get_feed")}),
        "/:slug".By(Ohkami::new((
            "/"
                .GET(|slug: String| async move {
                    Text(format!("get_article {slug}"))
                })
                .PUT(|slug: String| async move {
                    Text(format!("put_article {slug}"))
                })
                .DELETE(|slug: String| async move {
                    Text(format!("delete_article {slug}"))
                }),
            "/comments"
                .POST(|slug: String| async move {
                    Text(format!("post_comments {slug}"))
                })
                .GET(|slug: String| async move {
                    Text(format!("get_comments {slug}"))
                }),
            "/comments/:id"
                .DELETE(|(slug, id): (String, usize)| async move {
                    Text(format!("delete_comment {slug} / {id}"))
                }),
            "/favorite"
                .POST(|slug: String| async move {
                    Text(format!("favorite_article {slug}"))
                })
                .DELETE(|slug: String| async move {
                    Text(format!("unfavorite_article {slug}"))
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
    use crate::FrontFang;

    fn N() -> &'static Mutex<usize> {
        static N: OnceLock<Mutex<usize>> = OnceLock::new();
        N.get_or_init(|| Mutex::new(0))
    }

    struct Increment;
    impl FrontFang for Increment {
        fn bite(&self, _: &mut Request) -> impl std::future::Future<Output = Result<(), Response>> + Send {
            *N().lock().unwrap() += 1;

            async {Ok(())}
        }
    }

    async fn h() -> &'static str {"h"}

    let o = Ohkami::with(Increment, (
        "/a"  .GET(h),
        "/a/b".GET(h),
    ));

//    let req = TestRequest::GET("/a");
//    o.oneshot(req).await;
//    assert_eq!(*N().lock().unwrap(), 1);
//
//    let req = TestRequest::GET("/a");
//    o.oneshot(req).await;
//    assert_eq!(*N().lock().unwrap(), 2);

    let req = TestRequest::GET("/a/b");
    o.oneshot(req).await;
    assert_eq!(*N().lock().unwrap(), 3);

    panic!()
}
