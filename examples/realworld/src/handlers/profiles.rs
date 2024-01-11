use ohkami::{Ohkami, Route, utils::typed::OK};
use crate::{fangs::Auth, models::{Profile, ProfileResponse}, errors::RealWorldError};


pub fn profiles_ohkami() -> Ohkami {
    Ohkami::with(Auth::default(), (
        "/:username"
            .GET(get_profile),
        "/:username/follow"
            .POST(follow)
            .DELETE(unfollow),
    ))
}

async fn get_profile(username: String) -> Result<OK<ProfileResponse>, RealWorldError> {
    todo!()
}

async fn follow(username: String) -> Result<OK<ProfileResponse>, RealWorldError> {
    todo!()
}

async fn unfollow(username: String) -> Result<OK<ProfileResponse>, RealWorldError> {
    todo!()
}
