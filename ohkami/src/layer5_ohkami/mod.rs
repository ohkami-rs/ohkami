pub(crate) mod builder;

use crate::{layer3_fang_handler::{Fang, Handlers}, layer4_router::TrieRouter};


pub struct Ohkami {
    pub(crate) routes: TrieRouter,
} impl Ohkami {
    pub(crate) fn new() -> Self {
        Self {
            routes: TrieRouter::new(),
        }
    }
}
