use ohkami_lib::CowSlice;

#[cfg(feature="sse")]
use ohkami_lib::Stream;

#[cfg(not(feature="rt_lambda"/* currently */))]
#[cfg(all(feature="ws", feature="__rt__"))]
use crate::ws::Session;


pub enum Content {
    None,

    Payload(CowSlice),

    #[cfg(feature="sse")]
    Stream(std::pin::Pin<Box<dyn Stream<Item = String> + Send>>),

    #[cfg(not(feature="rt_lambda"/* currently */))]
    #[cfg(all(feature="ws", feature="__rt__"))]
    WebSocket(Session),
}
const _: () = {
    impl Default for Content {
        fn default() -> Self {
            Self::None
        }
    }

    impl PartialEq for Content {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Content::None, Content::None) => true,

                (Content::Payload(p1), Content::Payload(p2)) => p1 == p2,

                _ => false
            }
        }
    }

    impl std::fmt::Debug for Content {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::None           => f.write_str("None"),

                Self::Payload(bytes) => f.write_str(&bytes.escape_ascii().to_string()),

                #[cfg(feature="sse")]
                Self::Stream(_)      => f.write_str("{stream}"),

                #[cfg(not(feature="rt_lambda"/* currently */))]
                #[cfg(all(feature="ws", feature="__rt__"))]
                Self::WebSocket(_)   => f.write_str("{websocket}"),
            }
        }
    }
};

impl Content {
    #[inline]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn take(&mut self) -> Content {
        std::mem::take(self)
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Payload(bytes) => Some(&bytes),
            _ => None
        }
    }
    pub fn into_bytes(self) -> Option<std::borrow::Cow<'static, [u8]>> {
        match self {
            Self::Payload(bytes) => Some(unsafe {bytes.into_cow_static_bytes_uncheked()}),
            _ => None
        }
    }
}

#[cfg(feature="rt_worker")]
impl Content {
    pub(crate) fn into_worker_response(self) -> ::worker::Response {
        match self {
            Self::None           => ::worker::Response::empty(),

            Self::Payload(bytes) => ::worker::Response::from_bytes(bytes.into()),

            #[cfg(feature="sse")]
            Self::Stream(stream) => ::worker::Response::from_stream({
                use {ohkami_lib::StreamExt, std::convert::Infallible};
                stream.map(Result::<_, Infallible>::Ok)
            }),

            #[cfg(feature="ws")]
            Self::WebSocket(ws)  => ::worker::Response::from_websocket(ws),
            
        }.expect("failed to convert Ohkami Response to Workers one")
    }
}
