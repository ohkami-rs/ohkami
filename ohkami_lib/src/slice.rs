use std::{borrow::Cow, ptr::NonNull};


/// A byte slice with **MANUALLY HANDLE** the *lifetime*
#[derive(Clone)]
pub struct Slice {
    head: NonNull<u8>,
    size: usize,
}
impl Slice {
    /// SAFETY: `head` is **NOT** null pointer
    #[inline(always)] pub unsafe fn new_unchecked(head: *const u8, size: usize) -> Self {
        Self {
            head: NonNull::new(head as *mut _).unwrap_unchecked(),
            size,
        }
    }
    #[inline(always)] pub unsafe fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            head: NonNull::new(bytes.as_ptr() as *mut _).unwrap_unchecked(),
            size: bytes.len(),
        }
    }
    
    #[inline(always)] pub const unsafe fn as_bytes<'b>(&self) -> &'b [u8] {
        std::slice::from_raw_parts(self.head.as_ptr(), self.size)
    }
}
const _: () = {
    unsafe impl Send for Slice {}
    unsafe impl Sync for Slice {}
};


pub enum CowSlice {
    Ref(Slice),
    Own(Vec<u8>),
}
impl CowSlice {
    #[inline(always)] pub unsafe fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Own(vec)   => &vec,
            Self::Ref(slice) => unsafe {slice.as_bytes()},
        }
    }
    #[inline(always)] pub unsafe fn from_request_cow_bytes<'req>(cow_bytes: std::borrow::Cow<'req, [u8]>) -> Self {
        match cow_bytes {
            std::borrow::Cow::Borrowed(slice) => Self::Ref(Slice::from_bytes(slice)),
            std::borrow::Cow::Owned(vec)      => Self::Own(vec),
        }
    }
    #[inline] pub unsafe fn extend(&mut self, bytes: &[u8]) {
        match self {
            Self::Own(vec)   => vec.extend_from_slice(bytes),
            Self::Ref(slice) => {
                let mut vec: Vec<_> = slice.as_bytes().into();
                vec.extend_from_slice(bytes);
                *self = Self::Own(vec);
            }
        }
    }
}
const _: () = {
    impl AsRef<[u8]> for CowSlice {
        fn as_ref(&self) -> &[u8] {
            match self {
                Self::Own(vec)   => vec,
                Self::Ref(slice) => unsafe {slice.as_bytes()},
            }
        }
    }

    impl From<Cow<'static, str>> for CowSlice {
        fn from(cow: Cow<'static, str>) -> Self {
            match cow {
                Cow::Borrowed(s)   => Self::Ref(unsafe {Slice::from_bytes(s.as_bytes())}),
                Cow::Owned(string) => Self::Own(string.into_bytes()),
            }
        }
    }

    impl PartialEq for CowSlice {
        fn eq(&self, other: &Self) -> bool {
            unsafe {self.as_bytes() == other.as_bytes()}
        }
    }
    impl Eq for CowSlice {}

    use std::hash::Hash;
    impl Hash for CowSlice {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            unsafe {self.as_bytes()}.hash(state)
        }
    }
};
