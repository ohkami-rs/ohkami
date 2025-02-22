use std::{borrow::Cow, ptr::NonNull};


/// A byte slice with **MANUALLY HANDLE** the *lifetime*
#[derive(Clone)]
pub struct Slice {
    head: NonNull<u8>,
    size: usize,
}
impl Slice {
    /// SAFETY: `head` is non-null pointer
    #[inline(always)] pub unsafe fn new_unchecked(head: *const u8, size: usize) -> Self {
        Self {
            head: unsafe {NonNull::new_unchecked(head as _)},
            size,
        }
    }
    #[inline(always)] pub const fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            head: unsafe {NonNull::new_unchecked(bytes.as_ptr() as _)},
            size: bytes.len(),
        }
    }
    
    #[inline(always)] pub const unsafe fn as_bytes<'b>(&self) -> &'b [u8] {
        unsafe {std::slice::from_raw_parts(self.head.as_ptr(), self.size)}
    }
}
const _: () = {
    unsafe impl Send for Slice {}
    unsafe impl Sync for Slice {}

    impl PartialEq for Slice {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            unsafe {self.as_bytes() == other.as_bytes()}
        }
    }
    impl Eq for Slice {}

    impl PartialOrd for Slice {
        #[inline]
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            unsafe {PartialOrd::partial_cmp(self.as_bytes(), other.as_bytes())}
        }
    }
    impl Ord for Slice {
        #[inline]
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            unsafe {Ord::cmp(self.as_bytes(), other.as_bytes())}
        }
    }

    impl std::hash::Hash for Slice {
        #[inline]
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            std::hash::Hash::hash(unsafe {self.as_bytes()}, state)
        }
    }
};


#[derive(Clone)]
pub enum CowSlice {
    Ref(Slice),
    Own(Box<[u8]>),
}
impl CowSlice {
    #[inline(always)]
    pub unsafe fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Own(array) => &array,
            Self::Ref(slice) => unsafe {slice.as_bytes()},
        }
    }

    #[cold]
    pub unsafe fn extend_from_slice(&mut self, bytes: &[u8]) {
        match self {
            Self::Own(array) => {
                let current = std::mem::take(array);                
                let mut appended = {
                    let mut current = current.into_vec();
                    current.extend_from_slice(bytes);
                    current.into_boxed_slice()
                };
                std::mem::swap(&mut appended, array)
            }
            Self::Ref(slice) => {
                let mut vec: Vec<_> = unsafe {slice.as_bytes()}.into();
                vec.extend_from_slice(bytes);
                *self = Self::Own(vec.into_boxed_slice());
            }
        }
    }

    #[inline]
    pub unsafe fn into_cow_static_bytes_uncheked(self) -> Cow<'static, [u8]> {
        match self {
            Self::Own(array) => Cow::Owned(array.into()),
            Self::Ref(slice) => Cow::Borrowed(unsafe {slice.as_bytes()}),
        }
    }
}
const _: () = {
    impl AsRef<[u8]> for CowSlice {
        #[inline]
        fn as_ref(&self) -> &[u8] {
            match self {
                Self::Own(array) => array,
                Self::Ref(slice) => unsafe {slice.as_bytes()},
            }
        }
    }
    impl std::ops::Deref for CowSlice {
        type Target = [u8];
        #[inline]
        fn deref(&self) -> &Self::Target {
            self.as_ref()
        }
    }

    impl From<Cow<'static, str>> for CowSlice {
        #[inline]
        fn from(cow: Cow<'static, str>) -> Self {
            match cow {
                Cow::Borrowed(s)   => Self::Ref(Slice::from_bytes(s.as_bytes())),
                Cow::Owned(string) => Self::Own(string.into_bytes().into_boxed_slice()),
            }
        }
    }
    impl From<Cow<'static, [u8]>> for CowSlice {
        #[inline]
        fn from(cow: Cow<'static, [u8]>) -> Self {
            match cow {
                Cow::Borrowed(s) => Self::Ref(Slice::from_bytes(s)),
                Cow::Owned(vec)  => Self::Own(vec.into_boxed_slice()),
            }
        }
    }
    impl From<Vec<u8>> for CowSlice {
        #[inline]
        fn from(vec: Vec<u8>) -> Self {
            Self::Own(vec.into_boxed_slice())
        }
    }
    impl Into<Vec<u8>> for CowSlice {
        #[inline]
        fn into(self) -> Vec<u8> {
            match self {
                Self::Own(array) => array.into(),
                Self::Ref(slice) => Vec::from(unsafe {slice.as_bytes()}),
            }
        }
    }
    impl From<&'static [u8]> for CowSlice {
        #[inline]
        fn from(slice: &'static [u8]) -> Self {
            Self::Ref(Slice::from_bytes(slice))
        }
    }

    impl PartialEq for CowSlice {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            unsafe {self.as_bytes() == other.as_bytes()}
        }
    }
    impl Eq for CowSlice {}

    use std::hash::Hash;
    impl Hash for CowSlice {
        #[inline]
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            unsafe {self.as_bytes()}.hash(state)
        }
    }
};
