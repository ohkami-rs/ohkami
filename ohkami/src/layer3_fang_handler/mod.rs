mod fang; pub use fang::*;
mod handler; pub use handler::*;


// #[cfg(test)] mod __ {
//     enum Response {
//         Ok,
//         Err,
//     }
// 
//     fn __() -> Response {
//         let r: Result<usize, std::io::Error> = Ok(42);
//         let _: usize = r?;
//     }
// }
// 