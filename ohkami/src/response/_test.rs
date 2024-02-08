#![cfg(feature="testing")]

use crate::Response;

#[crate::__rt__::test]
async fn test_response_into_bytes() {
    macro_rules! assert_bytes_eq {
        ($actual:expr, $expected:expr) => {
            {
                if &$actual != &$expected {
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

    let res = Response::NoContent();
    let res_bytes = res.into_bytes();
    assert_bytes_eq!(&res_bytes, b"\
        HTTP/1.1 204 No Content\r\n\
        \r\n\
    ");

    let mut res = Response::NoContent();
    res.headers.set().Server("ohkami");
    let res_bytes = res.into_bytes();
    assert_bytes_eq!(&res_bytes, b"\
        HTTP/1.1 204 No Content\r\n\
        Server: ohkami\r\n\
        \r\n\
    ");

    let res = Response::NotFound();
    let res_bytes = res.into_bytes();
    assert_bytes_eq!(&res_bytes, b"\
        HTTP/1.1 404 Not Found\r\n\
        Content-Length: 0\r\n\
        \r\n\
    ");

    let mut res = Response::NotFound();
    res.headers.set().Server("ohkami");
    let res_bytes = res.into_bytes();
    assert_bytes_eq!(&res_bytes, b"\
        HTTP/1.1 404 Not Found\r\n\
        Content-Length: 0\r\n\
        Server: ohkami\r\n\
        \r\n\
    ");
}
