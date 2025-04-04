use ohkami::prelude::*;

struct Options {
    omit_dot_html: bool,
    serve_dotfiles: bool,
    etag: Option<fn(&std::fs::File) -> String>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            omit_dot_html: false,
            serve_dotfiles: false,
            etag: None,
        }
    }
}

fn ohkami(Options { omit_dot_html, serve_dotfiles, etag }: Options) -> Ohkami {
    Ohkami::new((
        "/".Dir("./public")
            .omit_extensions(if omit_dot_html {&["html"]} else {&[]})
            .serve_dotfiles(serve_dotfiles)
            .etag(etag),
    ))
}

#[tokio::main]
async fn main() {
    ohkami(Default::default()).howl("0.0.0.0:3000").await
}

#[cfg(test)]
mod test {
    use super::*;
    use ohkami::testing::*;

    #[tokio::test]
    async fn test_default() {
        let t = ohkami(Default::default()).test();

        // dotfiles are not served
        {
            let req = TestRequest::GET("/.env.sample");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 404);
        }

        // .js is served as text/javascript
        {
            let req = TestRequest::GET("/index.js");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.content("text/javascript"), Some(include_str!("../public/index.js").as_bytes()));
        }

        // .html is served as text/html with extension
        {
            let req = TestRequest::GET("/index.html");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.content("text/html"), Some(include_str!("../public/index.html").as_bytes()));

            let req = TestRequest::GET("/about.html");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.content("text/html"), Some(include_str!("../public/about.html").as_bytes()));

            let req = TestRequest::GET("/blog/index.html");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.content("text/html"), Some(include_str!("../public/blog/index.html").as_bytes()));
        }
    }

    #[tokio::test]
    async fn test_omit_dot_html() {
        let t = ohkami(Options {
            omit_dot_html: true,
            ..Default::default()
        }).test();

        // dotfiles are not served
        {
            let req = TestRequest::GET("/.env.sample");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 404);
        }

        // .js is served as text/javascript
        {
            let req = TestRequest::GET("/index.js");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.content("text/javascript"), Some(include_str!("../public/index.js").as_bytes()));
        }

        // .html is served as text/html without extension and index.html is served at /
        {
            let req = TestRequest::GET("/"); // <---
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.content("text/html"), Some(include_str!("../public/index.html").as_bytes()));

            let req = TestRequest::GET("/about"); // <---
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.content("text/html"), Some(include_str!("../public/about.html").as_bytes()));

            let req = TestRequest::GET("/blog"); // <---
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.content("text/html"), Some(include_str!("../public/blog/index.html").as_bytes()));
        }
    }

    #[tokio::test]
    async fn test_serve_dotfiles() {
        let t = ohkami(Options {
            serve_dotfiles: true,
            ..Default::default()
        }).test();

        // !!! dotfiles are served !!!
        {
            let req = TestRequest::GET("/.env.sample");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.content("application/octet-stream"), Some(include_str!("../public/.env.sample").as_bytes()));
        }

        // .js is served as text/javascript
        {
            let req = TestRequest::GET("/index.js");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.content("text/javascript"), Some(include_str!("../public/index.js").as_bytes()));
        }

        // .html is served as text/html with extension
        {
            let req = TestRequest::GET("/index.html");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.content("text/html"), Some(include_str!("../public/index.html").as_bytes()));

            let req = TestRequest::GET("/about.html");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.content("text/html"), Some(include_str!("../public/about.html").as_bytes()));

            let req = TestRequest::GET("/blog/index.html");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.content("text/html"), Some(include_str!("../public/blog/index.html").as_bytes()));
        }
    }

    #[tokio::test]
    async fn test_precompressed() {
        // `../public/sub.js` and `../public/blog/second.html` have pre-compressed version:
        // 
        // - `../public/sub.js.gz`
        // - `../public/sub.js.br`
        // - `../public/blog/second.html.gz`
        // 
        // They are used for response when the client accepts gzip or brotli encoding.
        // Then, brotli version is smaller than gzip version, so it is preferred
        // when the client accepts it.
        let t = ohkami(Default::default()).test();

        // sub.js.br is used for requests that accept brotli
        {
            let req = TestRequest::GET("/sub.js");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.header("Content-Encoding"), Some("br"));
            assert_eq!(res.content("text/javascript"), Some(include_bytes!("../public/sub.js.br").as_slice()));
        }

        // sub.js.gz is used for requests that does not accept brotli and accepts gzip
        {
            let req = TestRequest::GET("/sub.js")
                .header("Accept-Encoding", "gzip, deflate");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.header("Content-Encoding"), Some("gzip"));
            assert_eq!(res.content("text/javascript"), Some(include_bytes!("../public/sub.js.gz").as_slice()));
        }

        {
            let req = TestRequest::GET("/sub.js");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.header("Content-Encoding"), Some("br"));
            assert_eq!(res.content("text/javascript"), Some(include_bytes!("../public/sub.js.br").as_slice()));
        }

        // fallback to .js if request does not accept all prepared compressions
        {
            let req = TestRequest::GET("/sub.js")
                .header("Accept-Encoding", "gzip;q=0, br;q=0");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.header("Content-Encoding"), None);
            assert_eq!(res.content("text/javascript"), Some(include_str!("../public/sub.js").as_bytes()));

            let req = TestRequest::GET("/sub.js")
                .header("Accept-Encoding", "deflate, identity");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.header("Content-Encoding"), None);
            assert_eq!(res.content("text/javascript"), Some(include_str!("../public/sub.js").as_bytes()));
        }

        // fallback to .js if no precompressed version is found
        {
            let req = TestRequest::GET("/index.js");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.header("Content-Encoding"), None);
            assert_eq!(res.content("text/javascript"), Some(include_str!("../public/index.js").as_bytes()));
        }

        // respond with 406 Not Acceptable if
        // no precompressed version is accepted
        // and the request explicitly forbids identity
        {
            let req = TestRequest::GET("/sub.js")
                .header("Accept-Encoding", "deflate, identity;q=0");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 406);
            assert_eq!(res.header("Content-Encoding"), None);
            assert_eq!(res.content("text/javascript"), None);

            let req = TestRequest::GET("/sub.js")
                .header("Accept-Encoding", "*;q=0");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 406);
            assert_eq!(res.header("Content-Encoding"), None);
            assert_eq!(res.content("text/javascript"), None);

            let req = TestRequest::GET("/index.js")
                .header("Accept-Encoding", "identity;q=0");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 406);
            assert_eq!(res.header("Content-Encoding"), None);
            assert_eq!(res.content("text/javascript"), None);
        }

        // precompressed files in subdirectory are used with no problem
        {
            let req = TestRequest::GET("/blog/second.html");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.header("Content-Encoding"), Some("gzip"));
            assert_eq!(res.content("text/html"), Some(include_bytes!("../public/blog/second.html.gz").as_slice()));

            let req = TestRequest::GET("/blog/second.html")
                .header("Accept-Encoding", "deflate, gzip;q=0");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.header("Content-Encoding"), None);
            assert_eq!(res.content("text/html"), Some(include_str!("../public/blog/second.html").as_bytes()));

            let req = TestRequest::GET("/blog/second.html")
                .header("Accept-Encoding", "br");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 200);
            assert_eq!(res.header("Content-Encoding"), None);
            assert_eq!(res.content("text/html"), Some(include_str!("../public/blog/second.html").as_bytes()));

            let req = TestRequest::GET("/blog/second.html")
                .header("Accept-Encoding", "gzip;q=0, identity;q=0");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 406);
            assert_eq!(res.header("Content-Encoding"), None);
            assert_eq!(res.content("text/html"), None);

            let req = TestRequest::GET("/blog/second.html")
                .header("Accept-Encoding", "*;q=0");
            let res = t.oneshot(req).await;
            assert_eq!(res.status().code(), 406);
            assert_eq!(res.header("Content-Encoding"), None);
            assert_eq!(res.content("text/html"), None);
        }
    }
}
