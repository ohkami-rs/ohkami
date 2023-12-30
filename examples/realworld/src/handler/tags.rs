use ohkami::{Ohkami, Route, Context, Response};


pub fn tags_ohkami() -> Ohkami {
    Ohkami::new((
        "/".GET(get),
    ))
}

async fn get(c: Context) -> Response {
    todo!()
}
