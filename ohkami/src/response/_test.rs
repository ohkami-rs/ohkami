#![cfg(any(feature="rt_tokio", feature="rt_async-std"))]
#![cfg(feature="testing")]

use crate::Response;

#[crate::__rt__::test]
async fn test_response_into_bytes() {
    macro_rules! assert_bytes_eq {
        ($actual:expr, $expected:expr) => {
            {
                if $actual != $expected {
                    panic!("\n\
                        [got]\n\
                        {}\n\
                        [expected]\n\
                        {}\n",
                        ($actual).escape_ascii(),
                        ($expected).escape_ascii(),
                    )
                }
            }
        };
    }

    let __now__ = ::ohkami_lib::imf_fixdate(
        std::time::Duration::from_secs(crate::utils::unix_timestamp())
    );

    let res = Response::NoContent();
    let res_bytes = res.into_bytes().await;
    assert_bytes_eq!(res_bytes, format!("\
        HTTP/1.1 204 No Content\r\n\
        Date: {__now__}\r\n\
        \r\n\
    ").into_bytes());

    let mut res = Response::NoContent();
    res.headers.set().Server("ohkami");
    let res_bytes = res.into_bytes().await;
    assert_bytes_eq!(res_bytes, format!("\
        HTTP/1.1 204 No Content\r\n\
        Server: ohkami\r\n\
        Date: {__now__}\r\n\
        \r\n\
    ").into_bytes());

    let res = Response::NotFound();
    let res_bytes = res.into_bytes().await;
    assert_bytes_eq!(res_bytes, format!("\
        HTTP/1.1 404 Not Found\r\n\
        Date: {__now__}\r\n\
        Content-Length: 0\r\n\
        \r\n\
    ").into_bytes());

    let mut res = Response::NotFound();
    res.headers.set()
        .Server("ohkami")
        .custom("Hoge-Header", "Something-Custom");
    let res_bytes = res.into_bytes().await;
    assert_bytes_eq!(res_bytes, format!("\
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
    let res_bytes = res.into_bytes().await;
    assert_bytes_eq!(res_bytes, format!("\
        HTTP/1.1 404 Not Found\r\n\
        Server: ohkami\r\n\
        Date: {__now__}\r\n\
        Content-Length: 0\r\n\
        Hoge-Header: Something-Custom\r\n\
        Set-Cookie: id=42; Path=/; SameSite=Lax\r\n\
        Set-Cookie: name=John; Path=/where; SameSite=Strict\r\n\
        \r\n\
    ").into_bytes());
}
