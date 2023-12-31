use ohkami::{Ohkami, Route, Context, Response};
use ohkami::utils::Payload;
use serde::Deserialize;
use crate::fangs::Auth;


pub fn user_ohkami() -> Ohkami {
    Ohkami::with(Auth::default(), (
        "/"
            .GET(get_current)
            .POST(update),
    ))
}

async fn get_current(c: Context) -> Response {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct UpdateRequest {
    email:    Option<String>,
    username: Option<String>,
    password: Option<String>,
    image:    Option<String>,
    bio:      Option<String>,
}
async fn update(c: Context) -> Response {
    todo!()
}
