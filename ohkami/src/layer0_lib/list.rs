pub(crate) struct List<T, const CAPACITY: usize> {
    pub(crate) list: [Option<T>; CAPACITY],
    pub(crate) next: usize,
}

macro_rules! prepare_capacity {
    ($cap:literal: [ $( $none:ident ),* ]) => {
        impl<T> List<T, $cap> {
            pub(crate) fn new() -> Self {
                Self {
                    list: [ $( $none ),* ],
                    next: 0,
                }
            }
        }
    };
} const _: () = {
    prepare_capacity!(2: [None, None]);
    prepare_capacity!(4: [None, None, None, None]);
    prepare_capacity!(32: [
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None
    ]);
};

impl<T, const CAPACITY: usize> List<T, CAPACITY> {
    pub(crate) fn append(&mut self, element: T) {
        if self.next == CAPACITY {
            eprintln!("Buffer over flow");
        } else {
            self.list[self.next].replace(element);
            self.next += 1;
        }
    }
}
