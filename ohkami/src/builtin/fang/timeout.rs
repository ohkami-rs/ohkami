use std::time::Duration;


/// # Builtin fang for timeout config
///
/// **NOTE**:
/// **CANNOT** used as a **global fang** (fang you pass in `Ohkami::howl_with`) !
/// 
/// <br>
/// 
/// Set timeout of request handling when a request was handled by that `Ohkami`.
/// 
/// In ohkami, timeout configuration is
/// **NOT** applied for *global fangs*,
/// just for *ordinary fangs and a handler*.
/// 
/// Default timeout: `42secs`
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::builtin::fang::Timeout;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() {
///     Ohkami::with(Timeout(Duration::from_secs(10)), (
///         "/hello/:sleep".GET(sleeping_hello),
///     )).howl("0.0.0.0:3000").await
/// }
/// 
/// async fn sleeping_hello(sleep: u64) -> &'static str {
///     tokio::time::sleep(Duration::from_secs(sleep)).await;
///     
///     "Hello, I was sleeping ):"
/// }
/// ```
/// ---
#[derive(Clone, Copy)]
pub struct Timeout(pub Duration);

/* `Timeout`
    _**MUST**_ be consumed in router configuration process append
    _**MUST NOT**_ be actually called in any handling process
*/

const _: () = {
    impl Default for Timeout {
        fn default() -> Self {
            Timeout(Duration::from_secs(42))
        }
    }
};
