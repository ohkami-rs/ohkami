use std::future::Future;
use super::{Fang, IntoFrontFang};


pub mod public {
    use std::future::Future;
    use crate::layer3_fang_handler::IntoFrontFang;

    pub struct Fangs;
    impl Fangs {
        pub fn front<F: IntoFrontFang<Args, Fut>, Args, Fut:Future<Output = ()>>(self, fang: F) -> super::Fangs {
            let mut fangs = super::Fangs::new();
            fangs = fangs.front(fang);
            fangs
        }
    }
}

pub struct Fangs(
    Vec<Fang>
);

impl Fangs {
    fn new() -> Self {
        Self(Vec::new())
    }
    pub fn front<F: IntoFrontFang<Args, Fut>, Args, Fut:Future<Output = ()>>(mut self, fang: F) -> Self {
        self.0.push(fang.into_fang());
        self
    }
}
