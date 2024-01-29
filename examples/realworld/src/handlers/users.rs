use std::borrow::Cow;
use ohkami::{Ohkami, Route, typed::{Created, OK}};
use crate::{
    models::User,
    models::response::UserResponse,
    models::request::{LoginRequest, LoginRequestUser, RegisterRequest},
    errors::RealWorldError,
    config::{pool, self},
    db,
};


pub fn users_ohkami() -> Ohkami {
    Ohkami::new((
        "/login"
            .POST(login),
        "/"
            .POST(register),
    ))
}

async fn login(
    LoginRequest {
        user: LoginRequestUser { email, password },
    }: LoginRequest<'_>,
) -> Result<OK<UserResponse>, RealWorldError> {
    let hased_password = db::hash_password(password)?;

    let u = sqlx::query_as!(db::UserEntity, r#"
        SELECT id, email, name, bio, image_url
        FROM users AS u
        WHERE
            u.email    = $1 AND
            u.password = $2
    "#, email, hased_password.as_str())
        .fetch_one(pool()).await
        .map_err(RealWorldError::DB)?;

    Ok(OK(u.into_user_response()))
}

async fn register(
    RegisterRequest { username, email, password }: RegisterRequest<'_>,
) -> Result<Created<UserResponse>, RealWorldError> {
    let already_exists = sqlx::query!(r#"
        SELECT EXISTS (
            SELECT id
            FROM users AS u
            WHERE
                u.name = $1
        )
    "#, username)
        .fetch_one(pool()).await
        .map_err(RealWorldError::DB)?
        .exists.unwrap();
    if already_exists {
        return Err(RealWorldError::FoundUnexpectedly(Cow::Owned(
            format!("User of name = '{username}' & email = '{email}' is already exists")
        )))
    }

    let hased_password = db::hash_password(password)?;

    let new_user_id = sqlx::query!(r#"
        INSERT INTO
            users  (email, name, password)
            VALUES ($1,    $2,   $3)
        RETURNING id
    "#, email, username, hased_password.as_str())
        .fetch_one(pool()).await
        .map_err(RealWorldError::DB)?
        .id;

    Ok(Created(UserResponse {
        user: User {
            email: email.into(),
            jwt:   config::issue_jwt_for_user_of_id(new_user_id),
            name:  username.into(),
            bio:   None,
            image: None,
        },
    }))
}
