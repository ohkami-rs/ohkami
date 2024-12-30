/// returning `(next_section, remaining <starting with '/'>)` or `(path, empty)`
#[inline] pub(super) fn split_next_section(
    path: &[u8]
) -> (&[u8], &[u8]) {
    let ptr = path.as_ptr();
    let len = path.len();
    for i in 0..len {
        if &b'/' == unsafe {path.get_unchecked(i)} {
            return unsafe {(
                std::slice::from_raw_parts(ptr,        i),
                std::slice::from_raw_parts(ptr.add(i), len - i),
            )}
        }
    }; (path, b"")
}


#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct ID(usize);
impl ID {
    pub(super) fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static ID: AtomicUsize = AtomicUsize::new(1);
        Self(ID.fetch_add(1, Ordering::Relaxed))
    }
}
impl std::fmt::Debug for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature="DEBUG")]
pub(super) struct DebugSimpleOption<'option, T: std::fmt::Debug>(
    pub(super) &'option Option<T>
);
#[cfg(feature="DEBUG")]
impl<'option, T: std::fmt::Debug> std::fmt::Debug for DebugSimpleOption<'option, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(t) => write!(f, "Some({t:?})"),
            None    => f.write_str("None")
        }
    }
}

#[cfg(feature="DEBUG")]
pub(super) struct DebugSimpleIterator<I: Iterator<Item: std::fmt::Debug> + Clone>(
    pub(super) I
);
#[cfg(feature="DEBUG")]
impl<I: Iterator<Item: std::fmt::Debug> + Clone> std::fmt::Debug for DebugSimpleIterator<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&{
            let mut buf = String::new();
            buf.push('[');
            for item in self.0.clone() {
                buf.push_str(&format!("{item:?}"));
                buf.push(',');
            }
            if buf.ends_with(',') {buf.pop();}
            buf.push(']');
            buf
        })
    }
}
