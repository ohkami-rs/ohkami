#![cfg(feature="__rt__")]

mod util;
pub(crate) mod segments;

#[cfg(feature="__rt__")]
pub(crate) mod base;
#[cfg(feature="__rt__")]
pub(crate) mod r#final;
