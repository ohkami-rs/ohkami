use crate::{router::Router, fang::Fangs, handler::Handler};

pub struct Ohkami {
    router: Router,
}

impl Ohkami {
    #[inline] pub fn new<const N: usize>(handlers: [Handler; N]) -> Self {
        let mut router = Router::new();
        for handler in handlers {
            router.register(handler)
        }
        Self { router }
    }
    #[inline] pub fn with<const N: usize>(fangs: Fangs, handlers: [Handler; N]) -> Self {
        let mut router = Router::new();
        for handler in handlers {
            router.register(handler)
        }
        router.apply(fangs);
        Self { router }
    }

    pub async fn howl(self) -> crate::Result<()> {
        
    }
}
