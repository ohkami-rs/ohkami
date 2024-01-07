use ohkami::{Ohkami, Route, http::JSON};
use crate::{fangs::Auth, models::{Profile, ProfileResponse}};


pub fn profiles_ohkami() -> Ohkami {
    Ohkami::with(Auth::default(), (
        "/:username"
            .GET(get_profile),
        "/:username/follow"
            .POST(follow)
            .DELETE(unfollow),
    ))
}

async fn get_profile(username: String) -> JSON<ProfileResponse> {
    todo!()
}

async fn follow(username: String) -> JSON<ProfileResponse> {
    todo!()
}

async fn unfollow(username: String) -> JSON<ProfileResponse> {
    todo!()
}
