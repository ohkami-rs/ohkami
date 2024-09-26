#![cfg(any(feature="rt_tokio", feature="rt_async-std"))]

use crate::Response;


macro_rules! assert_bytes_eq {
    ($res:expr, $expected:expr) => {
        {
            let mut res_bytes = Vec::new();
            $res.send(&mut res_bytes).await;

            if res_bytes != $expected {
                panic!("\n\
                    [got]\n\
                    {}\n\
                    [expected]\n\
                    {}\n",
                    (res_bytes).escape_ascii(),
                    ($expected).escape_ascii(),
                )
            }
        }
    };
}

#[crate::__rt__::test]
async fn test_response_into_bytes() {
    let __now__ = ::ohkami_lib::imf_fixdate(
        std::time::Duration::from_secs(crate::util::unix_timestamp())
    );

    let res = Response::NoContent();
    assert_bytes_eq!(res, format!("\
        HTTP/1.1 204 No Content\r\n\
        Date: {__now__}\r\n\
        \r\n\
    ").into_bytes());

    let mut res = Response::NoContent();
    res.headers.set().Server("ohkami");
    assert_bytes_eq!(res, format!("\
        HTTP/1.1 204 No Content\r\n\
        Server: ohkami\r\n\
        Date: {__now__}\r\n\
        \r\n\
    ").into_bytes());

    let res = Response::NotFound();
    assert_bytes_eq!(res, format!("\
        HTTP/1.1 404 Not Found\r\n\
        Date: {__now__}\r\n\
        Content-Length: 0\r\n\
        \r\n\
    ").into_bytes());

    let mut res = Response::NotFound();
    res.headers.set()
        .Server("ohkami")
        .custom("Hoge-Header", "Something-Custom");
    assert_bytes_eq!(res, format!("\
        HTTP/1.1 404 Not Found\r\n\
        Server: ohkami\r\n\
        Date: {__now__}\r\n\
        Content-Length: 0\r\n\
        Hoge-Header: Something-Custom\r\n\
        \r\n\
    ").into_bytes());

    let mut res = Response::NotFound();
    res.headers.set()
        .Server("ohkami")
        .custom("Hoge-Header", "Something-Custom")
        .SetCookie("id", "42", |d|d.Path("/").SameSiteLax())
        .SetCookie("name", "John", |d|d.Path("/where").SameSiteStrict());
    assert_bytes_eq!(res, format!("\
        HTTP/1.1 404 Not Found\r\n\
        Server: ohkami\r\n\
        Date: {__now__}\r\n\
        Content-Length: 0\r\n\
        Hoge-Header: Something-Custom\r\n\
        Set-Cookie: id=42; Path=/; SameSite=Lax\r\n\
        Set-Cookie: name=John; Path=/where; SameSite=Strict\r\n\
        \r\n\
    ").into_bytes());

    let mut res = Response::NotFound().with_text("sample text");
    res.headers.set()
        .Server("ohkami")
        .custom("Hoge-Header", "Something-Custom")
        .SetCookie("id", "42", |d|d.Path("/").SameSiteLax())
        .SetCookie("name", "John", |d|d.Path("/where").SameSiteStrict());
    assert_bytes_eq!(res, format!("\
        HTTP/1.1 404 Not Found\r\n\
        Content-Type: text/plain; charset=UTF-8\r\n\
        Server: ohkami\r\n\
        Date: {__now__}\r\n\
        Content-Length: 11\r\n\
        Hoge-Header: Something-Custom\r\n\
        Set-Cookie: id=42; Path=/; SameSite=Lax\r\n\
        Set-Cookie: name=John; Path=/where; SameSite=Strict\r\n\
        \r\n\
        sample text\
    ").into_bytes());
}

#[cfg(feature="sse")]
#[crate::__rt__::test]
async fn test_stream_response() {
    let __now__ = ::ohkami_lib::imf_fixdate(
        std::time::Duration::from_secs(crate::util::unix_timestamp())
    );

    fn repeat_by<T, F: Fn(usize) -> T + Unpin>(
        n: usize,
        f: F
    ) -> impl ohkami_lib::Stream<Item = T> {
        struct Repeat<F> {
            f: F,
            n: usize,
            count: usize,
        } const _: () = {
            impl<T, F: Fn(usize) -> T + Unpin> ohkami_lib::Stream for Repeat<F> {
                type Item = T;
                fn poll_next(mut self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
                    if self.count < self.n {
                        let item = std::task::Poll::Ready(Some((self.f)(self.count)));
                        self.as_mut().count += 1;
                        item
                    } else {
                        std::task::Poll::Ready(None)
                    }
                }
            }
        };

        Repeat { f, n, count: 0 }
    }

    let res = Response::OK()
        .with_stream(
            repeat_by(3, |i| Result::<_, std::convert::Infallible>::Ok(
                format!("This is message#{i} !")
            ))
        )
        .with_headers(|h| h
            .Server("ohkami")
            .custom("is-stream", "true")
            .SetCookie("name", "John", |d|d.Path("/where").SameSiteStrict())
        );
    assert_bytes_eq!(res, format!("\
        HTTP/1.1 200 OK\r\n\
        Content-Type: text/event-stream\r\n\
        Cache-Control: no-cache, must-revalidate\r\n\
        Transfer-Encoding: chunked\r\n\
        Server: ohkami\r\n\
        Date: {__now__}\r\n\
        is-stream: true\r\n\
        Set-Cookie: name=John; Path=/where; SameSite=Strict\r\n\
        \r\n\
        1b\r\n\
        data: This is message#0 !\n\
        \n\
        \r\n\
        1b\r\n\
        data: This is message#1 !\n\
        \n\
        \r\n\
        1b\r\n\
        data: This is message#2 !\n\
        \n\
        \r\n\
        0\r\n\
        \r\n\
    ").into_bytes());

    let res = Response::OK()
        .with_stream(
            repeat_by(3, |i| Result::<_, std::convert::Infallible>::Ok(
                format!("This is message#{i}\nです")
            ))
        )
        .with_headers(|h| h
            .Server("ohkami")
            .custom("is-stream", "true")
            .SetCookie("name", "John", |d|d.Path("/where").SameSiteStrict())
        );
    assert_bytes_eq!(res, format!("\
        HTTP/1.1 200 OK\r\n\
        Content-Type: text/event-stream\r\n\
        Cache-Control: no-cache, must-revalidate\r\n\
        Transfer-Encoding: chunked\r\n\
        Server: ohkami\r\n\
        Date: {__now__}\r\n\
        is-stream: true\r\n\
        Set-Cookie: name=John; Path=/where; SameSite=Strict\r\n\
        \r\n\
        26\r\n\
        data: This is message#0\n\
        data: です\n\
        \n\
        \r\n\
        26\r\n\
        data: This is message#1\n\
        data: です\n\
        \n\
        \r\n\
        26\r\n\
        data: This is message#2\n\
        data: です\n\
        \n\
        \r\n\
        0\r\n\
        \r\n\
    ").into_bytes());
}
