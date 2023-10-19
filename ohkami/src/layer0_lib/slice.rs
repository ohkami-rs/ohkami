/// MANUALLY HANDLE the *lifetime*
#[derive(Clone, Copy)]
pub(crate) struct Slice {
    head: *const u8,
    size: usize,
} impl Slice {
    #[inline] pub(crate) unsafe fn from_bytes(bytes: &[u8]) -> Self {
        Self { head: bytes.as_ptr(), size: bytes.len() }
    }
    #[inline] pub(crate) unsafe fn into_bytes<'s>(self) -> &'s [u8] {
        std::slice::from_raw_parts(self.head, self.size)
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


impl Slice {
    #[inline] pub(crate) fn cow(self) -> CowSlice {
        CowSlice::Ref(self)
    }
}

impl CowSlice {
    #[inline] pub(crate) fn into_own(self) -> Self {
        match self {
            Self::Own(vec)   => Self::Own(vec),
            Self::Ref(slice) => Self::Own(unsafe {slice.into_bytes()}.to_vec()),
        }
    }
    #[inline] pub(crate) unsafe fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Own(vec)   => &vec,
            Self::Ref(slice) => unsafe {slice.into_bytes()},
        }
    }
}
