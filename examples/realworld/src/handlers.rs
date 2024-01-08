mod users;
mod user;
mod profiles;
mod articles;
mod tags;

use ohkami::{Ohkami, Route};
use crate::fangs::{LogRequest, LogResponse};


pub fn realworld_ohkami(

) -> Ohkami {
    Ohkami::with((LogRequest, LogResponse),
        "/api".By(Ohkami::new((
            "/users"   .By(users::users_ohkami()),
            "/user"    .By(user::user_ohkami()),
            "/profiles".By(profiles::profiles_ohkami()),
            "/articles".By(articles::articles_ohkami()),
            "/tags"    .By(tags::tags_ohkami()),
        ))
    ))
}
