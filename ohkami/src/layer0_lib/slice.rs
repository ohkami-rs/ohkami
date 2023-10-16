/// MANUALLY HANDLE the *lifetime*
#[derive(Clone, Copy)]
pub(crate) struct Slice {
    head: *const u8,
    size: usize,
} impl Slice {
    #[inline(always)] pub(crate) unsafe fn from_bytes(bytes: &[u8]) -> Self {
        Self { head: bytes.as_ptr(), size: bytes.len() }
    }
    #[inline(always)] pub(crate) unsafe fn into_bytes<'s>(self) -> &'s [u8] {
        std::slice::from_raw_parts(self.head, self.size)
    }
} const _: () = {
    unsafe impl Send for Slice {}
    unsafe impl Sync for Slice {}
};
