mod query;
pub use query::Query;

mod builtin;
pub use builtin::*;


#[cfg(feature="nightly")]
pub trait Schema {
    #[inline(always)]
    fn valid(&self) -> Result<(), impl std::fmt::Display> {
        Result::<(), &str>::Ok(())
    }
}
