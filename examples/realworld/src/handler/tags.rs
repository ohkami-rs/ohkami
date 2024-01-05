use ohkami::{Ohkami, Route, Response};


pub fn tags_ohkami() -> Ohkami {
    Ohkami::new((
        "/".GET(get),
    ))
}

async fn get() -> Response {
    todo!()
}
