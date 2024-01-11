use ohkami::{Ohkami, Route, utils::typed::OK};
use crate::{models::ListOfTagsResponse, errors::RealWorldError};


pub fn tags_ohkami() -> Ohkami {
    Ohkami::new((
        "/".GET(get),
    ))
}

async fn get() -> Result<OK<ListOfTagsResponse<'static>>, RealWorldError> {
    todo!()
}
