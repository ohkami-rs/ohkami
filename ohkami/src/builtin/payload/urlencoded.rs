use serde::{Serialize, Deserialize};
use ohkami_lib::serde_urlencoded;
use crate::typed::PayloadType;


/// Builtin `PayloadType` for `application/x-www-form-urlencoded` payloads.
/// 
/// _**note**_ : \
/// While non encoded value like `ohkami`
/// can be handled as `&'req str`, urlencoded value like
/// `%E3%81%8A%E3%81%8A%E3%81%8B%E3%81%BF` is automatically
/// decoded into `String` (then, it fails deserializing
/// if the corresponded field has type `&str`). \
/// So, if you have a field that's may or may not encoded,
/// `Cow<'req, str>` is the best choice.
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```
/// use ohkami::typed::Payload;
/// use ohkami::builtin::payload::URLEncoded; //
/// # use std::borrow::Cow;
/// 
/// #[Payload(URLEncoded/D)]
/// struct ExampleURLEncoded<'req> {
///     name:    &'req str,
///     profile: Cow<'req, str>,
/// }
/// 
/// 
/// use ohkami::typed::status::OK;
/// 
/// async fn handle_urlencoded(
///     ue: ExampleURLEncoded<'_>
/// ) -> OK {
///     println!(
///         "got example urlencoded: name = {:?}, profile = {:?}",
///         ue.name,
///         ue.profile,
///     );
/// 
///     OK(())
/// }
/// ```
/// ---
pub struct URLEncoded;

impl PayloadType for URLEncoded {
    const MIME_TYPE: &'static str = "application/x-www-form-urlencoded";

    #[inline]
    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl crate::serde::de::Error> {
        serde_urlencoded::from_bytes(bytes)
    }

    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, impl crate::serde::ser::Error> {
        serde_urlencoded::to_string(value).map(String::into_bytes)
    }
}


#[cfg(test)]
#[cfg(feature="testing")]
#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
mod test {
    use crate::{prelude::*, testing::*, typed::Payload};
    use super::URLEncoded;
    use std::borrow::Cow;


    #[derive(serde::Deserialize)]
    struct URLRequest<'req> {
        url: Cow<'req, str>,
    }
    impl<'req> Payload for URLRequest<'req> {
        type Type = URLEncoded;
    }

    async fn get_url(
        body: URLRequest<'_>,
    ) -> String {
        String::from(body.url)
    }

    #[crate::__rt__::test] async fn extract_urlencoded_request() {
        let t = Ohkami::new((
            "/".GET(get_url),
        )).test();

        {
            let req = TestRequest::GET("/");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::BadRequest);
        }

        {
            let req = TestRequest::GET("/").content(
                "application/x-www-form-urlencoded",
                b"url=https://scrapbox.io/nwtgck/Rust%E3%81%AEHyper_+_Rustls%E3%81%A7HTTPS%E3%82%B5%E3%83%BC%E3%83%90%E3%83%BC%E3%82%92%E7%AB%8B%E3%81%A6%E3%82%8B%E3%82%B7%E3%83%B3%E3%83%97%E3%83%AB%E3%81%AA%E4%BE%8B"
            );
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(),   Some("https://scrapbox.io/nwtgck/RustのHyper_+_RustlsでHTTPSサーバーを立てるシンプルな例"));
        }
    }
}
