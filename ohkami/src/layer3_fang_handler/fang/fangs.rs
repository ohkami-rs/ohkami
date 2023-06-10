use std::sync::{Arc, Mutex};
use crate::{Context};
use super::{Fang, IntoFang};


/// ```ignore
/// let my_fangs = Fangs
///     .before(log)
///     .before::<"/api/*">(authenticate);
/// ```
pub struct Fangs {
    fangs: Vec<(&'static str, Fang)>,
    pub before: BeforeFangs,
    // pub after: AfterFangs,
} impl Fangs {
    fn new(fangs: Vec<(&'static str, Fang)>) -> Self {
        let before = BeforeFangs(Arc::new(Mutex::new(&fangs)));
        Self {
            fangs,
            before: ,
        }
    }
}

pub struct BeforeFangs<const PATH: &'static str = "/*">(
    Arc<Mutex<Vec<(&'static str, Fang)>>>
); const _: () = {
    const _: (/* by */) = {
        impl<'c, const PATH: &'static str, F: IntoFang<(&'c Context,)>> FnOnce<(F,)> for BeforeFangs<PATH> {
            type Output = Fangs;
            extern "rust-call" fn call_once(self, (f,): (F,)) -> Self::Output {
                let mut this = self.0.lock().unwrap();
                (&mut this).fangs.push((PATH, f.into_fang()));
                *this
            }
        }
    };
};
