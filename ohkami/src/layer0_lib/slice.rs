// use std::ptr::NonNull;

use std::ptr::NonNull;

/// MANUALLY HANDLE the *lifetime*
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
            head: NonNull::new(head as *mut _),// Box::from_raw(head as *mut u8),//NonNull::new_unchecked(head as *mut u8),
            size,
        }
    }
    #[inline] pub(crate) unsafe fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            head: NonNull::new(bytes.as_ptr() as *mut _),//Box::from_raw(bytes.as_ptr() as *mut u8),// NonNull::new_unchecked(bytes.as_ptr() as *mut u8),
            size: bytes.len(),
        }
    }
    #[inline] pub(crate) unsafe fn as_bytes(&self) -> &[u8] {
        self.head.map(|p| std::slice::from_raw_parts(
            p.as_ptr(),
            self.size,
        )).unwrap_or(&[])
        // std::slice::from_raw_parts(
        //     self.head.un,//Box::into_raw(self.head),//self.head,//.as_ptr(),
        //     self.size,
        // )
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
            Self::Ref(slice) => unsafe {slice.as_bytes()},
        }
    }
}
