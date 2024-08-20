mod json;
pub use json::JSON;

mod multipart;
pub use multipart::{Multipart, File};

mod urlencoded;
pub use urlencoded::URLEncoded;

mod text;
pub use text::Text;

mod html;
pub use html::HTML;

mod query;
pub use query::Query;
