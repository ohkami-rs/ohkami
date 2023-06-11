use std::{ops::{Index, IndexMut, Deref, Add}, collections::HashMap, any::{TypeId, Any}, sync::{OnceLock, Mutex}, cell::OnceCell};
use crate::{layer3_fang_handler::{FrontFang, Fang}, Context, layer0_lib::List};


pub(crate) mod public {
    /// <br/>
    /// 
    /// ```ignore
    /// async fn main() -> Result<()> {
    ///     let api_fangs = Fangs
    ///         .front(log)
    ///         .front(auth);
    /// 
    ///     let api_ohkami = Ohkami[api_fangs](
    ///         "/users"
    ///             .POST(create_user),
    ///         "/users/:id"
    ///             .GET(get_user_by_id),
    ///             .PATCH(update_user),
    ///     );
    /// 
    ///     let root_fangs = Fangs
    ///         .front(log);
    /// 
    ///     Ohkami[root_fangs](
    ///         "/hc".GET(health_check),
    ///         "/api".by(api_ohkami),
    ///     ).howl(":3000").await
    /// }
    /// ```
    #[allow(non_upper_case_globals)]
    pub const Ohkami: super::Ohkami = super::Ohkami::new();
}


pub(crate) const FANGS_LIMIT: usize = 8;
static FANGS_MAP: OnceLock<Mutex<HashMap<TypeId, Fang>>> = OnceLock::new();

#[must_use]
#[derive(Clone, Copy)]
pub struct Ohkami { fangs: List<TypeId, FANGS_LIMIT> }
impl Ohkami {
    const fn new() -> Self {
        Self { fangs: List::<TypeId, FANGS_LIMIT>::new() }
    }
}

const _: () = { // conflicting...
    // macro_rules! indexing_fang {
    //     ($( $args:ty ),*) => {$(
    //         impl<'c, F: IntoFang<$args>> Index<F> for Ohkami {
    //             type Output = Ohkami;
    //             fn index(&self, _: F) -> &Self::Output {unimplemented!()}
    //         }
    //         impl<'c, F: IntoFang<$args> + 'static> IndexMut<F> for Ohkami {
    //             fn index_mut(&mut self, new_fang: F) -> &mut Self::Output {
    //                 let new_fang_id = new_fang.type_id();
    //                 FANGS_MAP.get_or_init(|| Mutex::new(HashMap::new()))
    //                     .lock()
    //                     .unwrap()
    //                     .insert(new_fang_id, new_fang.into_fang());
    //                 self.fangs.append(new_fang_id);
    //                 self
    //             }
    //         }
    //     )*};
    // } indexing_fang! { (&'c Context,), (&'c mut Context,),  }
    // 

    // impl<'c, F: IntoFang<impl ToString>> Index<F> for Ohkami {
    //     type Output = Ohkami;
    //     fn index(&self, _: F) -> &Self::Output {unimplemented!()}
    // }
    // impl<'c, F: IntoFang<impl ToString> + 'static> IndexMut<F> for Ohkami {
    //     fn index_mut(&mut self, new_fang: F) -> &mut Self::Output {
    //         let new_fang_id = new_fang.type_id();
    //         FANGS_MAP.get_or_init(|| Mutex::new(HashMap::new()))
    //             .lock()
    //             .unwrap()
    //             .insert(new_fang_id, new_fang.into_fang());
    //         self.fangs.append(new_fang_id);
    //         self
    //     }
    // }

    // #[cfg(test)] #[allow(unused)] fn __(
    //     f1: impl IntoFang<(&'static Context,)>,
    //     f2: impl IntoFang<(&'static Context,)>,
    // ) {
    //     let o = public::Ohkami;
    //     // let o = o[f1][f2];
    // }
};

const _: () = {
//    impl FnMut<>
};


