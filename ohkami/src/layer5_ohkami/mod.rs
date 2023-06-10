pub(crate) mod builder;

use crate::layer3_fang_handler::{Fang, Handlers};


pub struct Ohkami {
    fangs:    Vec<Fang>,
    handlers: Vec<Handlers>,
} impl Ohkami {
    pub(crate) fn new() -> Self {
        Self {
            fangs: Vec::new(),
            handlers: Vec::new(),
        }
    }
}
