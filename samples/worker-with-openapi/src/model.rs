use ohkami::serde::{Serialize, Deserialize};

#[cfg(feature="openapi")]
use ohkami::openapi::Schema;

// #[derive(Serialize, Deserialize)]
// #[cfg_attr(feature="openapi", derive(Schema))]
// struct User {
//     id:      i32,
//     name:    String,
//     country: Option<String>,
//     age:     Option<u8>,
// }
