use std::borrow::Cow;
use crate::{Request};


/// Implmented for structs you put
/// 
/// - `#[Queries]`
/// - `#[Headers]`
/// - `#[Payload(JSON)]`
/// - `#[Payload(Form)]`
/// - `#[Payload(URLEncoded)]`
/// 
/// <br/>
/// 
/// And, you can manually implement for any structs that can be extracted from request：
/// 
/// ```ignore
/// struct HasPayload(bool);
/// 
/// impl FromRequest for HasPayload {
///     fn parse(req: &Request) -> Result<Self, Cow> {
///         Ok(Self(
///             req.payload.is_some()
///         ))
///     }
/// }
/// ```
pub trait FromRequest: Sized {
    fn parse(req: &Request) -> Result<Self, Cow<'static, str>>;
}

pub trait FromBuffer: Sized {
    fn parse(buffer: &[u8]) -> Result<Self, Cow<'static, str>>;
} const _: () = {
    // impl FromBuffer for &str {
    //     fn parse(buffer: &[u8]) -> Result<Self, Cow<'static, str>> {
    //         // SAFETY:
    //         // - This str refers to `buffer` in `Request`
    //         // - And, this str is visible to user **ONLY IN** the handler
    //         //   that is handling the request
    //         Ok(unsafe {std::mem::transmute(
    //             std::str::from_utf8(buffer)
    //                 .map_err(move |e| Cow::Owned(e.to_string()))?
    //         )})
    //     }
    // }

    impl FromBuffer for String {
        fn parse(buffer: &[u8]) -> Result<Self, Cow<'static, str>> {
            Ok(String::from(
                std::str::from_utf8(buffer)
                    .map_err(move |e| Cow::Owned(e.to_string()))?
            ))
        }
    }

    macro_rules! unsigned_integers {
        ($( $unsigned_int:ty ),*) => {
            $(
                impl FromBuffer for $unsigned_int {
                    fn parse(buffer: &[u8]) -> Result<Self, Cow<'static, str>> {
                        #[cfg(debug_assertions)]
                        println!("[FromBuffer::parse] buffer: {:?}", buffer);

                        if buffer.is_empty() {return Err(Cow::Borrowed("Expected a number nut found an empty string"))}
                        match buffer[0] {
                            b'-' => Err(Cow::Borrowed("Expected non-negative number but found negetive one")),
                            b'0' => Err(Cow::Borrowed("Expected a number but it starts with '0'")),
                            _ => {
                                let mut value: $unsigned_int = 0;
                                for d in buffer {
                                    match d {
                                        b'0'..=b'9' => value = value * 10 + (*d - b'0') as $unsigned_int,
                                        _ => return Err(Cow::Borrowed("Expected a number but it contains a non-digit charactor"))
                                    }
                                }
                                Ok(value)
                            }
                        }
                    }
                }
            )*
        };
    } unsigned_integers! { u8, u16, u32, u64, u128, usize }
};


