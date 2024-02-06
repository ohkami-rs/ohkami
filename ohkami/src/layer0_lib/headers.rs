/// Passed to `{Request/Response}.headers.set().Name( 〜 )` and
/// append `value` to the header
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// # use ohkami::prelude::*;
/// use ohkami::append;
/// 
/// struct AppendServer;
/// impl IntoFang for AppendServer {
///     fn into_fang(self) -> Fang {
///         Fang::back(|res: &mut Response| {
///             res.headers.set()
///                 .Server(append("ohkami"));
///         })
///     }
/// }
/// ```
pub fn append(value: impl Into<std::borrow::Cow<'static, str>>) -> Append {
    Append(value.into())
}

pub struct Append(pub(crate) std::borrow::Cow<'static, str>);
