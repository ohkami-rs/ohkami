use ohkami::{Ohkami, Route, Context, Response};
use crate::fangs::Auth;


pub fn profiles_ohkami() -> Ohkami {
    Ohkami::with(Auth, (
        "/:username"
            .GET(get_profile),
        "/:username/follow"
            .POST(follow)
            .DELETE(unfollow),
    ))
}

async fn get_profile(c: Context, username: String) -> Response {
    todo!()
}

async fn follow(c: Context, username: String) -> Response {
    todo!()
}

async fn unfollow(c: Context, username: String) -> Response {
    todo!()
}
