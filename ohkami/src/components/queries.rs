const QUERY_PARAM_MAX: usize = 4;

pub(crate) struct QueritParams<'buf>(
    [Option<(&'buf str, &'buf str)>; 4]
);

const _: (/* QueryParams impls */) = {
    impl<'buf> QueritParams<'buf> {
        #[inline] pub(crate) fn new() -> Self {
            Self([None, None, None, None])
        }

        #[inline] pub(crate) fn insert(&mut self, (key, value): (&'buf str, &'buf str)) {
            
        }
    }
};
