use std::borrow::Cow;


pub struct Append(pub(crate) Cow<'static, str>);

/// Passed to `{Request/Response}.headers.set().Name( 〜 )` and
/// append `value` to the header.
/// 
/// Here appended values are combined by `,`.
/// 
/// ---
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::header::append;
/// 
/// #[derive(Clone)]
/// struct AppendServer(&'static str);
/// impl FangAction for AppendServer {
///     async fn back<'b>(&'b self, res: &'b mut Response) {
///         res.headers.set()
///             .Server(append(self.0));
///     }
/// }
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         AppendServer("ohkami"),
///         
///         "/".GET(|| async {"Hello, append!"})
/// 
///     )).howl("localhost:3000").await
/// }
/// ```
#[inline]
pub fn append(value: impl Into<std::borrow::Cow<'static, str>>) -> Append {
    Append(value.into())
}
