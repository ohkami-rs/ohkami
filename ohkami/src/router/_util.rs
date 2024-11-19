/// returning `(next_section, remaining <starting with '/'>)`
#[inline] pub(super) fn split_next_section(
    path: &[u8]
) -> Option<(&[u8], &[u8])> {
    let ptr = path.as_ptr();
    let len = path.len();

    for i in 0..len {
        if &b'/' == unsafe {path.get_unchecked(i)} {
            return Some(unsafe {(
                std::slice::from_raw_parts(ptr,        i),
                std::slice::from_raw_parts(ptr.add(i), len - i),
            )})
        }
    }; None
}

