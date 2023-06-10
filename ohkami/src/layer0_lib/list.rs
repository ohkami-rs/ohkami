use std::mem::MaybeUninit;


pub(crate) struct List<T, const CAPACITY: usize> {
    pub(crate) list: [MaybeUninit<T>; CAPACITY],
    pub(crate) next: usize,
}

macro_rules! prepare_capacity {
    ($cap_name:ident: [ $( $uninit:ident ),* ]) => {
        impl<T> List<T, $cap_name> {
            #[inline(always)] pub(crate) const fn new() -> Self {
                Self {
                    list: [ $( MaybeUninit::$uninit() ),* ],
                    next: 0,
                }
            }
        }
    };
} const _: () = {
    use crate::{PATH_PARAMS_LIMIT, QUERIES_LIMIT, HEADERS_LIMIT, layer5_ohkami::builder::FANGS_LIMIT};
    
    prepare_capacity!(PATH_PARAMS_LIMIT: [uninit, uninit]);
    prepare_capacity!(QUERIES_LIMIT: [uninit, uninit, uninit, uninit]);
    prepare_capacity!(FANGS_LIMIT: [
        uninit, uninit, uninit, uninit, uninit, uninit, uninit, uninit
    ]);
    prepare_capacity!(HEADERS_LIMIT: [
        uninit, uninit, uninit, uninit, uninit, uninit, uninit, uninit,
        uninit, uninit, uninit, uninit, uninit, uninit, uninit, uninit,
        uninit, uninit, uninit, uninit, uninit, uninit, uninit, uninit,
        uninit, uninit, uninit, uninit, uninit, uninit, uninit, uninit
    ]);
};

impl<T, const CAPACITY: usize> List<T, CAPACITY> {
    #[inline(always)] pub(crate) fn append(&mut self, element: T) {
        if self.next == CAPACITY {
            panic!("Buffer over flow");
        } else {
            self.list[self.next].write(element);
            self.next += 1;
        }
    }
}


const _: () = {
    impl Copy for List<::std::any::TypeId, {crate::layer5_ohkami::builder::FANGS_LIMIT}> {}
    impl Clone for List<::std::any::TypeId, {crate::layer5_ohkami::builder::FANGS_LIMIT}> {
        fn clone(&self) -> Self {
            let mut cloned = List::<::std::any::TypeId, {crate::layer5_ohkami::builder::FANGS_LIMIT}>::new();
            if self.next == 0 {return cloned}

            for elem in &self.list {
                cloned.append(unsafe {
                    elem
                        .assume_init_ref()
                        .clone()
                });
            }
            cloned
        }
    }
};

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
                if unsafe {self.list[i].assume_init_ref() != other.list[i].assume_init_ref()} {
                    return false
                }
            }
            true
        }
    }
};
