use super::Ohkami;
use crate::layer3_fang_handler::{Handlers, ByAnother};


const _: (/* 1 item */) = {
    impl FnOnce<(Handlers,)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (handlers1,): (Handlers,)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(handlers1)
            }
        }
    }
    impl FnOnce<(ByAnother,)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (another1,): (ByAnother,)) -> Self::Output {
            Ohkami{ routes: self.routes
                .merge_another(another1)
            }
        }
    }
};

const _: (/* 2 items */) = {
    impl FnOnce<(Handlers, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (handlers1, handlers2): (Handlers, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(handlers1)
                .register_handlers(handlers2)
            }
        }
    }

    impl FnOnce<(Handlers, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (handlers1, another1): (Handlers, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(handlers1)
                .merge_another(another1)
            }
        }
    }
    impl FnOnce<(ByAnother, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (another1, handlers1): (ByAnother, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(handlers1)
                .merge_another(another1)
            }
        }
    }
    impl FnOnce<(ByAnother, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (another1, another2): (ByAnother, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .merge_another(another2)
                .merge_another(another1)
            }
        }
    }
};

const _: (/* 3 items */) = {
    impl FnOnce<(Handlers, Handlers, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (handlers1, handlers2, handlers3): (Handlers, Handlers, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(handlers1)
                .register_handlers(handlers2)
                .register_handlers(handlers3)
            }
        }
    }

    impl FnOnce<(Handlers, Handlers, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (handlers1, handlers2, another1): (Handlers, Handlers, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(handlers1)
                .register_handlers(handlers2)
                .merge_another(another1)
            }
        }
    }
    impl FnOnce<(Handlers, ByAnother, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (handlers1, another1, handlers2): (Handlers, ByAnother, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(handlers1)
                .merge_another(another1)
                .register_handlers(handlers2)
            }
        }
    }
    impl FnOnce<(ByAnother, Handlers, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (another1, handlers1, handlers2): (ByAnother, Handlers, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .merge_another(another1)
                .register_handlers(handlers1)
                .register_handlers(handlers2)
            }
        }
    }

    impl FnOnce<(Handlers, ByAnother, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (handlers1, another1, another2): (Handlers, ByAnother, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(handlers1)
                .merge_another(another1)
                .merge_another(another2)
            }
        }
    }
    impl FnOnce<(ByAnother, Handlers, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (another1, handlers1, another2): (ByAnother, Handlers, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(handlers1)
                .merge_another(another1)
                .merge_another(another2)
            }
        }
    }
    impl FnOnce<(ByAnother, ByAnother, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (another1, another2, handlers1): (ByAnother, ByAnother, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(handlers1)
                .merge_another(another1)
                .merge_another(another2)
            }
        }
    }
    
    impl FnOnce<(ByAnother, ByAnother, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (another1, another2, another3): (ByAnother, ByAnother, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .merge_another(another1)
                .merge_another(another2)
                .merge_another(another3)
            }
        }
    }
};

const _: (/* 4 items */) = {
    impl FnOnce<(Handlers, Handlers, Handlers, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (h1, h2, h3, h4): (Handlers, Handlers, Handlers, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .register_handlers(h2)
                .register_handlers(h3)
                .register_handlers(h4)
            }
        }
    }

    impl FnOnce<(ByAnother, Handlers, Handlers, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (a1, h1, h2, h3): (ByAnother, Handlers, Handlers, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .register_handlers(h2)
                .register_handlers(h3)
                .merge_another(a1)
            }
        }
    }
    impl FnOnce<(Handlers, ByAnother, Handlers, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (h1, a1, h2, h3): (Handlers, ByAnother, Handlers, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .register_handlers(h2)
                .register_handlers(h3)
                .merge_another(a1)
            }
        }
    }
    impl FnOnce<(Handlers, Handlers, ByAnother, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (h1, h2, a1, h3): (Handlers, Handlers, ByAnother, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .register_handlers(h2)
                .register_handlers(h3)
                .merge_another(a1)
            }
        }
    }
    impl FnOnce<(Handlers, Handlers, Handlers, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (h1, h2, h3, a1): (Handlers, Handlers, Handlers, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .register_handlers(h2)
                .register_handlers(h3)
                .merge_another(a1)
            }
        }
    }

    impl FnOnce<(ByAnother, ByAnother, Handlers, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (a1, a2, h1, h2): (ByAnother, ByAnother, Handlers, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .register_handlers(h2)
                .merge_another(a2)
                .merge_another(a1)
            }
        }
    }
    impl FnOnce<(ByAnother, Handlers, ByAnother, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (a1, h1, a2, h2): (ByAnother, Handlers, ByAnother, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .register_handlers(h2)
                .merge_another(a2)
                .merge_another(a1)
            }
        }
    }
    impl FnOnce<(ByAnother, Handlers, Handlers, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (a1, h1, h2, a2): (ByAnother, Handlers, Handlers, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .register_handlers(h2)
                .merge_another(a2)
                .merge_another(a1)
            }
        }
    }
    impl FnOnce<(Handlers, ByAnother, ByAnother, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (h1, a1, a2, h2): (Handlers, ByAnother, ByAnother, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .register_handlers(h2)
                .merge_another(a2)
                .merge_another(a1)
            }
        }
    }
    impl FnOnce<(Handlers, ByAnother, Handlers, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (h1, a1, h2, a2): (Handlers, ByAnother, Handlers, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .register_handlers(h2)
                .merge_another(a2)
                .merge_another(a1)
            }
        }
    }
    impl FnOnce<(Handlers, Handlers, ByAnother, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (h1, h2, a1, a2): (Handlers, Handlers, ByAnother, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .register_handlers(h2)
                .merge_another(a2)
                .merge_another(a1)
            }
        }
    }

    impl FnOnce<(ByAnother, ByAnother, ByAnother, Handlers)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (a1, a2, a3, h1): (ByAnother, ByAnother, ByAnother, Handlers)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .merge_another(a2)
                .merge_another(a1)
                .merge_another(a3)
            }
        }
    }
    impl FnOnce<(ByAnother, ByAnother, Handlers, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (a1, a2, h1, a3): (ByAnother, ByAnother, Handlers, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .merge_another(a2)
                .merge_another(a1)
                .merge_another(a3)
            }
        }
    }
    impl FnOnce<(ByAnother, Handlers, ByAnother, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (a1, h1, a2, a3): (ByAnother, Handlers, ByAnother, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .merge_another(a2)
                .merge_another(a1)
                .merge_another(a3)
            }
        }
    }
    impl FnOnce<(Handlers, ByAnother, ByAnother, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (h1, a1, a2, a3): (Handlers, ByAnother, ByAnother, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .register_handlers(h1)
                .merge_another(a2)
                .merge_another(a1)
                .merge_another(a3)
            }
        }
    }

    impl FnOnce<(ByAnother, ByAnother, ByAnother, ByAnother)> for Ohkami {
        type Output = Ohkami;
        extern "rust-call" fn call_once(self, (a1, a2, a3, a4): (ByAnother, ByAnother, ByAnother, ByAnother)) -> Self::Output {
            Ohkami{ routes: self.routes
                .merge_another(a2)
                .merge_another(a1)
                .merge_another(a3)
                .merge_another(a4)
            }
        }
    }
};
