use std::{ops::{Index, IndexMut, Deref, Add}, collections::HashMap, any::{TypeId, Any}, sync::{OnceLock, Mutex, LazyLock}, cell::OnceCell};
use crate::{layer3_fang_handler::{IntoFrontFang, Fang, Fangs}, Context, layer0_lib::List};


mod without_fangs {
    use std::{ops::{Index, IndexMut}, any::Any, sync::MutexGuard};
    use crate::{layer3_fang_handler::{Fangs, Handlers, ByAnother}};

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
    ///             .GET(get_user_by_id)
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
    pub struct Ohkami;

    const _: (/* add Fangs */) = {
        impl FnOnce<(Fangs,)> for Ohkami {
            type Output = super::with_fangs::Ohkami;
            extern "rust-call" fn call_once(self, (fangs,): (Fangs,)) -> Self::Output {
                super::with_fangs::Ohkami {fangs}
            }
        }
    };

    // TODO!!!!
    // const _: (/* only Handlerses */) = {
    //     impl FnOnce<(Handlers,)> for Ohkami {
    //         type Output = crate::layer5_ohkami::Ohkami;
    //         extern "rust-call" fn call_once(self, (handlers1,): (Handlers,)) -> Self::Output {
    //             crate::layer5_ohkami::Ohkami {fangs: vec![],
    //                 handlers: vec![handlers1]
    //             }
    //         }
    //     }
    //     impl FnOnce<(Handlers, Handlers)> for Ohkami {
    //         type Output = crate::layer5_ohkami::Ohkami;
    //         extern "rust-call" fn call_once(self, (handlers1, handlers2): (Handlers, Handlers)) -> Self::Output {
    //             crate::layer5_ohkami::Ohkami {fangs: vec![],
    //                 handlers: vec![handlers1, handlers2]
    //             }
    //         }
    //     }
    //     impl FnOnce<(Handlers, Handlers, Handlers)> for Ohkami {
    //         type Output = crate::layer5_ohkami::Ohkami;
    //         extern "rust-call" fn call_once(self, (handlers1, handlers2, handlers3): (Handlers, Handlers, Handlers)) -> Self::Output {
    //             crate::layer5_ohkami::Ohkami {fangs: vec![],
    //                 handlers: vec![handlers1, handlers2, handlers3]
    //             }
    //         }
    //     }
    //     impl FnOnce<(Handlers, Handlers, Handlers, Handlers)> for Ohkami {
    //         type Output = crate::layer5_ohkami::Ohkami;
    //         extern "rust-call" fn call_once(self, (handlers1, handlers2, handlers3, handlers4): (Handlers, Handlers, Handlers, Handlers)) -> Self::Output {
    //             crate::layer5_ohkami::Ohkami {fangs: vec![],
    //                 handlers: vec![handlers1, handlers2, handlers3, handlers4]
    //             }
    //         }
    //     }
    // };
// 
    // TODO!!!!!
    const _: (/* with ByAnother */) = {
        // impl FnOnce<(ByAnother,)> for Ohkami {
        //     type Output = crate::layer5_ohkami::Ohkami;
        //     extern "rust-call" fn call_once(self, (by,): (ByAnother,)) -> Self::Output {
        //         crate::layer5_ohkami::Ohkami {fangs: vec![],
        //             handlers: vec![handlers]
        //         }
        //     }
        // }
    };
}

mod with_fangs {
    use std::any::TypeId;
    use crate::layer3_fang_handler::Fangs;

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
    ///             .GET(get_user_by_id)
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
    pub struct Ohkami {
        pub(super) fangs: Fangs,
    }
    
}
