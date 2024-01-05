use ohkami::{Ohkami, Route, Response};
use crate::fangs::Auth;


pub fn profiles_ohkami() -> Ohkami {
    Ohkami::with(Auth::default(), (
        "/:username"
            .GET(get_profile),
        "/:username/follow"
            .POST(follow)
            .DELETE(unfollow),
    ))
}

async fn get_profile(username: String) -> Response {
    todo!()
}

async fn follow(username: String) -> Response {
    todo!()
}

async fn unfollow(username: String) -> Response {
    todo!()
}
