use ohkami::{Ohkami, Route, Response};
use ohkami::{Fang, IntoFang};

struct SetServer;
impl IntoFang for SetServer {
    fn into_fang(self) -> Fang {
        Fang(|res: &mut Response| {
            res.headers.set()
                .Server("ohkami");
        })
    }
}

#[tokio::main]
async fn main() {
    // Use `with` to give
    // fangs for your Ohkami...
    let o = Ohkami::with((SetServer,), (
        "/".GET(|| async {
            "Hello!"
        }),
    ));

    use ohkami::{testing::*, Status};
    let req = TestRequest::GET("/")
        .header("Host", "localhost:5000")
        .header("User-Agent", "curl/7.81.0")
        .header("Accept", "*/*");
    let res = o.oneshot(req).await;
    assert_eq!(res.status(), Status::OK);
    assert_eq!(res.text(), Some("Hello!"));

    //o.howl(5000).await
}