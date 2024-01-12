use ohkami::{Ohkami, Route, utils::Payload, typed::OK};
use crate::{fangs::Auth, models::{User, UserResponse}, errors::RealWorldError};


pub fn user_ohkami() -> Ohkami {
    Ohkami::with(Auth::default(), (
        "/"
            .GET(get_current_user)
            .POST(update),
    ))
}

async fn get_current_user() -> Result<OK<UserResponse>, RealWorldError> {
    todo!()
}

#[Payload(JSOND)]
struct UpdateRequest {
    email:    Option<String>,
    username: Option<String>,
    password: Option<String>,
    image:    Option<String>,
    bio:      Option<String>,
}

async fn update() -> Result<OK<UserResponse>, RealWorldError> {
    todo!()
}
