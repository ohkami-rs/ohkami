pub(crate) struct List<T, const CAPACITY: usize> {
    pub(crate) list: [Option<T>; CAPACITY],
    pub(crate) next: usize,
}

macro_rules! prepare_capacity {
    ($cap_name:ident: [ $( $none:ident ),* ]) => {
        impl<T> List<T, $cap_name> {
            #[inline(always)] pub(crate) const fn new() -> Self {
                Self {
                    list: [ $( $none ),* ],
                    next: 0,
                }
            }
        }
    };
} const _: () = {
    use crate::{QUERIES_LIMIT, HEADERS_LIMIT};
    
    // prepare_capacity!(2: [None, None]);
    prepare_capacity!(QUERIES_LIMIT: [None, None, None, None]);
    prepare_capacity!(HEADERS_LIMIT: [
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None
    ]);
};

impl<T, const CAPACITY: usize> List<T, CAPACITY> {
    #[inline(always)] pub(crate) fn append(&mut self, element: T) {
        if self.next == CAPACITY {
            panic!("Buffer over flow");
        } else {
            self.list[self.next].replace(element);
            self.next += 1;
        }
    }
}


#[cfg(test)]
const _: () = {
    impl<T: PartialEq, const CAPACITY: usize> PartialEq for List<T, CAPACITY> {
        fn eq(&self, other: &Self) -> bool {
            let n = self.next;
            if other.next != n {
                return false
            } else if n == 0 {
                return true
            }

            for i in 0..n {
                if self.list[i] != other.list[i] {
                    return false
                }
            }
            true
        }
    }
};
