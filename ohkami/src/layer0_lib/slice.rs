use std::{ptr::NonNull};

/// Slice but MANUALLY HANDLE the *lifetime*
#[derive(Clone)]
pub(crate) struct Slice {
    head: Option<NonNull<u8>>,
    size: usize,
} impl Slice {
    pub(crate) fn null() -> Self {
        Self {
            head: None,
            size: 0,
        }
    }

    #[inline] pub(crate) unsafe fn new(head: *const u8, size: usize) -> Self {
        Self {
            head: NonNull::new(head as *mut _),
            size,
        }
    }
    #[inline] pub(crate) unsafe fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            head: NonNull::new(bytes.as_ptr() as *mut _),
            size: bytes.len(),
        }
    }
    #[inline] pub(crate) unsafe fn as_bytes<'b>(&self) -> &'b [u8] {
        self.head.map(|p| std::slice::from_raw_parts(
            p.as_ptr(),
            self.size,
        )).unwrap_or(&[])
    }
} const _: () = {
    unsafe impl Send for Slice {}
    unsafe impl Sync for Slice {}
};

pub(crate) enum CowSlice {
    Ref(Slice),
    Own(Vec<u8>),
} impl CowSlice {
    #[inline] pub(crate) unsafe fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Own(vec)   => &vec,
            Self::Ref(slice) => unsafe {slice.as_bytes()},
        }
    }
    #[inline] pub(crate) unsafe fn append(&mut self, bytes: &[u8]) {
        match self {
            Self::Own(vec)   => vec.extend_from_slice(bytes),
            Self::Ref(slice) => unsafe {
                let mut this = slice.as_bytes().to_vec();
                this.extend_from_slice(bytes);
                *self = Self::Own(this)
            },
        }
    }
} const _: () = {
    impl AsRef<[u8]> for CowSlice {
        fn as_ref(&self) -> &[u8] {
            match self {
                Self::Own(vec)   => vec,
                Self::Ref(slice) => unsafe {slice.as_bytes()},
            }
        }
    }
};
#[cfg(test)] impl PartialEq for CowSlice {
    fn eq(&self, other: &Self) -> bool {
        unsafe {self.as_bytes() == other.as_bytes()}
    }
}
