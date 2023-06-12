use std::{ops::{Index, IndexMut, Deref, Add}, collections::HashMap, any::{TypeId, Any}, sync::{OnceLock, Mutex, LazyLock}, cell::OnceCell};
use crate::{layer3_fang_handler::{IntoFrontFang, Fang, Fangs}, Context, layer0_lib::List};


mod without_fangs {
    use std::{ops::{Index, IndexMut}, any::Any, sync::MutexGuard};
    use crate::{layer3_fang_handler::Fangs};

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

    const _: (/* fangs-indexing */) = {
        impl Index<Fangs> for Ohkami {
            type Output = super::with_fangs::Ohkami;
            fn index(&self, fangs: Fangs) -> &Self::Output {
                compile_error!("\
                    Ohkami[fangs] と書きたいというだけの理由で leak するのは\
                    さすがに無駄なコストすぎるきがする.\
                    \n\n\
                    Fangs なしなら
                    \n\
                    ```\n\
                    Ohkami(\n\
                    \0  \"/users\"\n\
                    \0      .POST(create_user),\n\
                    \0  \"/users/:id\"\n\
                    \0      .GET(get_user_by_id)\n\
                    \0      .PATCH(update_user),\n\
                    )\n\
                    ```\n\
                    で, Fangs をつけたければ
                    \n\
                    ````\n\
                    Ohkami(fangs)(\n\
                    \0  \"/users\"\n\
                    \0      .POST(create_user),\n\
                    \0  \"/users/:id\"\n\
                    \0      .GET(get_user_by_id)\n\
                    \0      .PATCH(update_user),\n\
                    )\n\
                    ```\n\
                    とする, でいいのでは
                ");
                Box::leak(Box::new(
                    super::with_fangs::Ohkami{ fangs }
                ))
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
