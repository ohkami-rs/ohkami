mod users;
mod user;
mod profiles;
mod articles;
mod tags;

use sqlx::PgPool;
use ohkami::{Ohkami, Route};
use crate::fangs::{LogRequest, LogResponse, ConnectionPool};


pub fn realworld_ohkami(
    pool: PgPool,
) -> Ohkami {
    Ohkami::with((LogRequest, LogResponse, ConnectionPool::from(pool)),
        "/api".By(Ohkami::new((
            "/users"   .By(users::users_ohkami()),
            "/user"    .By(user::user_ohkami()),
            "/profiles".By(profiles::profiles_ohkami()),
            "/articles".By(articles::articles_ohkami()),
            "/tags"    .By(tags::tags_ohkami()),
        ))
    ))
}
