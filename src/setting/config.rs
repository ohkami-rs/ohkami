use tracing_subscriber::fmt::SubscriberBuilder;
use crate::components::cors::CORS;

/// Configurations of `Server`. In current version, this holds
/// 
/// - `cors: CORS`,
/// - `log_subscribe: Option<SubscriberBuilder>`,
/// - `db_profile: DBprofile<'url>` (if feature = "sqlx")
/// 
/// Here, `log_subscribe`'s default value is
/// ```no_run
/// Some(tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG))
/// ```
/// When you'd like to customize this, add `tracing` and `tracing_subscriber` in your dependencies to write custom config like
/// ```no_run
/// fn main() -> Result<()> {
///     let config = Config {
///         log_subscribe: Some(
///             tracing_subscriber::fmt()
///                 .with_max_level(tracing::Level::TRACE)
///         ),
///         ..Default::default()
///     };
/// }
/// ```
pub struct Config<#[cfg(feature = "sqlx")] 'url> {
    pub cors: CORS,
    pub log_subscribe: Option<SubscriberBuilder>,

    #[cfg(feature = "sqlx")]
    pub db_profile: DBprofile<'url>,
}
#[cfg(not(feature = "sqlx"))]
impl Default for Config {
    fn default() -> Self {
        Self {
            cors:          CORS::default(),
            log_subscribe: Some(tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG)),
        }
    }
}
#[cfg(feature = "sqlx")]
impl<'url> Default for Config<'url> {
    fn default() -> Self {
        Self {
            cors:          CORS::default(),
            log_subscribe: Some(tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG)),
            db_profile:    DBprofile::default(),
        }
    }
}