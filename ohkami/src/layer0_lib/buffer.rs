use std::ops::Index;

const BUFFER_SIZE: usize = 1024;
pub(crate) type BufRange = std::ops::Range<usize>;


pub(crate) struct Buffer(
    [u8; BUFFER_SIZE]
);

const _: () = {
    impl Index<BufRange> for Buffer {
        type Output = str;
        #[inline(always)] fn index(&self, range: BufRange) -> &Self::Output {
            unsafe {
                std::str::from_utf8_unchecked(
                    &self.0[range]
                )
            }
        }
    }
    impl<'range> Index<&'range BufRange> for Buffer {
        type Output = str;
        #[inline(always)] fn index(&self, range: &'range BufRange) -> &Self::Output {
            unsafe {
                std::str::from_utf8_unchecked(
                    &self.0[range.start..range.end]
                )
            }
        }
    }
};
