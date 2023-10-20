// use std::ptr::NonNull;

/// MANUALLY HANDLE the *lifetime*
#[derive(Clone)]
pub(crate) struct Slice {
    pub(crate) head: Box<u8>,// NonNull<u8>,
    pub(crate) size: usize,
} impl Slice {
    #[inline] pub(crate) unsafe fn new(head: *const u8, size: usize) -> Self {
        Self {
            head: Box::from_raw(head as *mut u8),//NonNull::new_unchecked(head as *mut u8),
            size,
        }
    }
    #[inline] pub(crate) unsafe fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            head: Box::from_raw(bytes.as_ptr() as *mut u8),// NonNull::new_unchecked(bytes.as_ptr() as *mut u8),
            size: bytes.len(),
        }
    }
    #[inline] pub(crate) unsafe fn into_bytes<'s>(self) -> &'s [u8] {
        std::slice::from_raw_parts(
            Box::into_raw(self.head),//self.head,//.as_ptr(),
            self.size,
        )
    }
} const _: () = {
    unsafe impl Send for Slice {}
    unsafe impl Sync for Slice {}
};

pub(crate) enum CowSlice {
    Ref(Slice),
    Own(Vec<u8>),
}
#[cfg(test)] impl PartialEq for CowSlice {
    fn eq(&self, other: &Self) -> bool {
        unsafe {self.as_bytes() == other.as_bytes()}
    }
}

impl CowSlice {
    #[inline] pub(crate) unsafe fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Own(vec)   => &vec,
            Self::Ref(slice) => unsafe {slice.clone().into_bytes()},
        }
    }
}
