use crate::f;

pub struct Global<T> {
    value: std::sync::OnceLock<T>,
    init:  fn() -> T,
}
impl<T> Global<T> {
    #[inline] pub const fn new(init: fn()->T) -> Self {
        Self { value: std::sync::OnceLock::new(), init }
    }
}
const _: () = {
    impl<T> std::ops::Deref for Global<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            self.value.get_or_init(self.init)
        }
    }
};


#[cfg(test)]
mod __example__ {use super::Global;

    static S: Global<String> = Global::new(|| String::from("Hello, world!"));

    #[test]
    fn s_ref() {
        let s_ref: &String = &S;
        assert_eq!(s_ref, "Hello, world!");
    }
}
