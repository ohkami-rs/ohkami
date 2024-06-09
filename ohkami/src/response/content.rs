use ohkami_lib::CowSlice;

#[cfg(feature="sse")]
use ::futures_core::stream::BoxStream;


pub enum Content {
    None,

    Payload(CowSlice),

    #[cfg(feature="sse")]
    Stream(BoxStream<'static, Result<CowSlice, String>>),
} const _: () = {
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

                #[cfg(feature="sse")]
                (Content::Stream(_), Content::Stream(_)) => false,

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
            Self::None           => ::worker::Response::empty().unwrap(),

            Self::Payload(bytes) => ::worker::Response::from_bytes(bytes.into()).unwrap(),

            #[cfg(feature="sse")]
            Self::Stream(stream) => ::worker::Response::from_stream(stream).unwrap()
        }
    }
}
