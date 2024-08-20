pub mod status;

#[cfg(feature="sse")]
mod stream;
#[cfg(feature="sse")]
pub use stream::DataStream;
