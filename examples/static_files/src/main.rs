use ohkami::prelude::*;

struct Options {
    omit_dot_html: bool,
    serve_dotfiles: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            omit_dot_html: false,
            serve_dotfiles: false,
        }
    }
}

fn ohkami(Options { omit_dot_html, serve_dotfiles }: Options) -> Ohkami {
    Ohkami::new((
        "/".Dir("./public")
            .omit_extensions(if omit_dot_html {&["html"]} else {&[]})
            .serve_dotfiles(serve_dotfiles),
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
}
