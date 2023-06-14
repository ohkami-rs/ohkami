use std::{ops::{Index, IndexMut, Deref, Add}, collections::HashMap, any::{TypeId, Any}, sync::{OnceLock, Mutex, LazyLock}, cell::OnceCell};
use crate::{layer3_fang_handler::{IntoFrontFang, Fang, Fangs}, Context, layer0_lib::List};


mod without_fangs {
    use std::{ops::{Index, IndexMut}, any::Any, sync::MutexGuard};
    use async_std::path::Ancestors;

    use crate::{layer3_fang_handler::{Fangs, Handlers, ByAnother}, layer4_router::TrieRouter};

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

    // const _: (/* add Fangs */) = {
    //     impl FnOnce<(Fangs,)> for Ohkami {
    //         type Output = super::with_fangs::Ohkami;
    //         extern "rust-call" fn call_once(self, (fangs,): (Fangs,)) -> Self::Output {
    //             super::with_fangs::Ohkami {fangs}
    //         }
    //     }
    // };

    // TODO!!!!
    const _: (/* only Handlers */) = {
        impl FnOnce<(Handlers,)> for Ohkami {
            type Output = crate::layer5_ohkami::Ohkami;
            extern "rust-call" fn call_once(self, (handlers1,): (Handlers,)) -> Self::Output {
                let routes = TrieRouter::new()
                    .register_handlers(handlers1);
                crate::layer5_ohkami::Ohkami{ routes }
            }
        }
        impl FnOnce<(Handlers, Handlers)> for Ohkami {
            type Output = crate::layer5_ohkami::Ohkami;
            extern "rust-call" fn call_once(self, (handlers1, handlers2): (Handlers, Handlers)) -> Self::Output {
                let routes = TrieRouter::new()
                    .register_handlers(handlers1)
                    .register_handlers(handlers2);
                crate::layer5_ohkami::Ohkami{ routes }
            }
        }
        impl FnOnce<(Handlers, Handlers, Handlers)> for Ohkami {
            type Output = crate::layer5_ohkami::Ohkami;
            extern "rust-call" fn call_once(self, (handlers1, handlers2, handlers3): (Handlers, Handlers, Handlers)) -> Self::Output {
                let routes = TrieRouter::new()
                    .register_handlers(handlers1)
                    .register_handlers(handlers2)
                    .register_handlers(handlers3);
                crate::layer5_ohkami::Ohkami{ routes }
            }
        }
        impl FnOnce<(Handlers, Handlers, Handlers, Handlers)> for Ohkami {
            type Output = crate::layer5_ohkami::Ohkami;
            extern "rust-call" fn call_once(self, (handlers1, handlers2, handlers3, handlers4): (Handlers, Handlers, Handlers, Handlers)) -> Self::Output {
                let routes = TrieRouter::new()
                    .register_handlers(handlers1)
                    .register_handlers(handlers2)
                    .register_handlers(handlers3)
                    .register_handlers(handlers4);
                crate::layer5_ohkami::Ohkami{ routes }
            }
        }
    };

    // TODO!!!!!
    const _: (/* with ByAnother */) = {
        // 1
        impl FnOnce<(ByAnother,)> for Ohkami {
            type Output = crate::layer5_ohkami::Ohkami;
            extern "rust-call" fn call_once(self, (another1,): (ByAnother,)) -> Self::Output {
                let routes = TrieRouter::new()
                    .merge_another(another1);
                crate::layer5_ohkami::Ohkami { routes }
            }
        }

        // 2
        impl FnOnce<(ByAnother, Handlers)> for Ohkami {
            type Output = crate::layer5_ohkami::Ohkami;
            extern "rust-call" fn call_once(self, (another1, handlers1): (ByAnother, Handlers)) -> Self::Output {
                let routes = TrieRouter::new()
                    .merge_another(another1)
                    .register_handlers(handlers1);
                crate::layer5_ohkami::Ohkami { routes }
            }
        }
        impl FnOnce<(Handlers, ByAnother)> for Ohkami {
            type Output = crate::layer5_ohkami::Ohkami;
            extern "rust-call" fn call_once(self, (handlers1, another1): (Handlers, ByAnother)) -> Self::Output {
                let routes = TrieRouter::new()
                    .merge_another(another1)
                    .register_handlers(handlers1);
                crate::layer5_ohkami::Ohkami { routes }
            }
        }

        // 3
        impl FnOnce<(ByAnother, Handlers, Handlers)> for Ohkami {
            type Output = crate::layer5_ohkami::Ohkami;
            extern "rust-call" fn call_once(self, (another1, handlers1, handlers2): (ByAnother, Handlers, Handlers)) -> Self::Output {
                let routes = TrieRouter::new()
                    .merge_another(another1)
                    .register_handlers(handlers1)
                    .register_handlers(handlers2);
                crate::layer5_ohkami::Ohkami { routes }
            }
        }
        impl FnOnce<(Handlers, ByAnother, Handlers)> for Ohkami {
            type Output = crate::layer5_ohkami::Ohkami;
            extern "rust-call" fn call_once(self, (handlers1, another1, handlers2): (Handlers, ByAnother, Handlers)) -> Self::Output {
                let routes = TrieRouter::new()
                    .merge_another(another1)
                    .register_handlers(handlers1)
                    .register_handlers(handlers2);
                crate::layer5_ohkami::Ohkami { routes }
            }
        }
        impl FnOnce<(Handlers, Handlers, ByAnother)> for Ohkami {
            type Output = crate::layer5_ohkami::Ohkami;
            extern "rust-call" fn call_once(self, (handlers1, handlers2, another1): (Handlers, Handlers, ByAnother)) -> Self::Output {
                let routes = TrieRouter::new()
                    .merge_another(another1)
                    .register_handlers(handlers1)
                    .register_handlers(handlers2);
                crate::layer5_ohkami::Ohkami { routes }
            }
        }
        impl FnOnce<(ByAnother, ByAnother, Handlers)> for Ohkami {
            type Output = crate::layer5_ohkami::Ohkami;
            extern "rust-call" fn call_once(self, (another1, another2, handlers1): (ByAnother, ByAnother, Handlers)) -> Self::Output {
                let routes = TrieRouter::new()
                    .merge_another(another1)
                    .merge_another(another2)
                    .register_handlers(handlers1);
                crate::layer5_ohkami::Ohkami { routes }
            }
        }
        impl FnOnce<(ByAnother, Handlers, ByAnother)> for Ohkami {
            type Output = crate::layer5_ohkami::Ohkami;
            extern "rust-call" fn call_once(self, (another1, handlers1, another2): (ByAnother, Handlers, ByAnother)) -> Self::Output {
                let routes = TrieRouter::new()
                    .merge_another(another1)
                    .merge_another(another2)
                    .register_handlers(handlers1);
                crate::layer5_ohkami::Ohkami { routes }
            }
        }
        impl FnOnce<(Handlers, ByAnother, ByAnother)> for Ohkami {
            type Output = crate::layer5_ohkami::Ohkami;
            extern "rust-call" fn call_once(self, (handlers1, another1, another2): (Handlers, ByAnother, ByAnother)) -> Self::Output {
                let routes = TrieRouter::new()
                    .merge_another(another1)
                    .merge_another(another2)
                    .register_handlers(handlers1);
                crate::layer5_ohkami::Ohkami { routes }
            }
        }
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
