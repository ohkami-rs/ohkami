mod users;
mod user;
mod profiles;
mod articles;
mod tags;

use sqlx::PgPool;
use ohkami::{Ohkami, Route};
use crate::fangs::{ConnectionPool, Logger};


pub fn realworld_ohkami(
    pool: PgPool,
) -> Ohkami {
    Ohkami::new(
        "/api".By(Ohkami::with((
            Logger,
            ConnectionPool::from(pool),
        ), (
            "/users"   .By(users::users_ohkami()),
            "/user"    .By(user::user_ohkami()),
            "/profiles".By(profiles::profiles_ohkami()),
            "/articles".By(articles::articles_ohkami()),
            "/tags"    .By(tags::tags_ohkami()),
        ))
    ))
}
