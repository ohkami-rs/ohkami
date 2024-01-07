use ohkami::{Ohkami, Route, utils::JSON};
use crate::models::ListOfTagsResponse;


pub fn tags_ohkami() -> Ohkami {
    Ohkami::new((
        "/".GET(get),
    ))
}

async fn get() -> JSON<ListOfTagsResponse<'static>> {
    todo!()
}
