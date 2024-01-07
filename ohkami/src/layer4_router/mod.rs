#![allow(non_snake_case)]

mod trie;  pub(crate) use trie::TrieRouter;
mod radix; pub(crate) use radix::RadixRouter;


#[cfg(test)] mod test {
    use crate::prelude::*;
    use crate::http::*;
    use crate::testing::*;
    use crate::__rt__::test;

    fn my_ohkami() -> Ohkami {
        let health_ohkami = Ohkami::new((
            "/".GET(|| async {Text::OK("health_check")}),
        ));

        let profiles_ohkami = Ohkami::new((
            "/:username"
                .GET(|username: String| async  move {
                    Text::OK(format!("get_profile of user `{username}`"))
                }),
            "/:username/follow"
                .POST(|username: String| async move {
                    Text::OK(format!("follow_user `{username}`"))
                })
                .DELETE(|username: String| async move {
                    Text::OK(format!("unfollow_user `{username}`"))
                })
        ));

        let articles_ohkami = Ohkami::new((
            "/"
                .GET(|| async {Text::OK("get_article")})
                .POST(|| async {Text::OK("post_article")}),
            "/feed"
                .GET(|| async {Text::OK("get_feed")}),
            "/:slug".By(Ohkami::new((
                "/"
                    .GET(|slug: String| async move {
                        Text::OK(format!("get_slug {slug}"))
                    })
                    .PUT(|slug: String| async move {
                        Text::OK(format!("put_slug {slug}"))
                    })
                    .DELETE(|slug: String| async move {
                        Text::OK(format!("delete_slug {slug}"))
                    }),
                "/comments"
                    .POST(|slug: String| async move {
                        Text::OK(format!("post_comments {slug}"))
                    })
                    .GET(|slug: String| async move {
                        Text::OK(format!("get_comments {slug}"))
                    }),
                "/comments/:id"
                    .DELETE(|(slug, id): (String, usize)| async move {
                        Text::OK(format!("delete_comment {slug} / {id}"))
                    }),
                "/favorite"
                    .POST(|slug: String| async move {
                        Text::OK(format!("favorite_article {slug}"))
                    })
                    .DELETE(|slug: String| async move {
                        Text::OK(format!("unfavorite_article {slug}"))
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

    #[test] async fn test_router() {
        let t = my_ohkami();

        let req = TestRequest::GET("/health");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("health_check"));

        let req = TestRequest::GET("/api/profiles");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::NotFound);

        let req = TestRequest::GET("/api/profiles/123");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some(""));

        let req = TestRequest::GET("/health");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("health_check"));

        let req = TestRequest::GET("/health");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("health_check"));

        let req = TestRequest::GET("/health");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("health_check"));

        let req = TestRequest::GET("/health");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("health_check"));

        let req = TestRequest::GET("/health");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("health_check"));

        let req = TestRequest::GET("/health");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("health_check"));

        let req = TestRequest::GET("/health");
        let res = t.oneshot(req).await;
        assert_eq!(res.text(), Some("health_check"));

    }
}
