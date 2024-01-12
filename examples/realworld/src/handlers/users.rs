use std::borrow::Cow;
use serde::Deserialize;
use ohkami::{Ohkami, Route, utils::{Payload, JSON}, typed::{Created, OK}};
use crate::{models::{User, UserResponse}, errors::RealWorldError, config::{pool, self}};


pub fn users_ohkami() -> Ohkami {
    Ohkami::new((
        "/login"
            .POST(login),
        "/"
            .POST(register),
    ))
}

#[Payload(JSON)]
struct LoginRequest<'req> {
    user: LoginRequestUser<'req>,
}
impl<'req> Deserialize<'req> for LoginRequest<'req> {
    fn deserialize<D: serde::Deserializer<'req>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self {
            user: LoginRequestUser::deserialize(deserializer)?,
        })
    }
}
#[derive(Deserialize)]
struct LoginRequestUser<'req> {
    email:    &'req str,
    password: &'req str,
}

async fn login(body: LoginRequest<'_>) -> Result<OK<UserResponse>, RealWorldError> {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct RegisterRequest<'req> {
    username: &'req str,
    email:    &'req str,
    password: &'req str,
}

async fn register(
    RegisterRequest { username, email, password }: RegisterRequest<'_>,
) -> Result<Created<UserResponse>, RealWorldError> {
    let already_exists = sqlx::query!(r#"
        SELECT EXISTS (
            SELECT id
            FROM users AS u
            WHERE
                u.name  = $1 AND
                u.email = $2
        )
    "#, username, email).fetch_one(pool()).await
        .map_err(RealWorldError::DB)?
        .exists.unwrap();
    if already_exists {
        return Err(RealWorldError::FoundUnexpectedly(Cow::Owned(
            format!("User of name = '{username}' & email = '{email}' is already exists")
        )))
    }

    let new_user_id = sqlx::query!(r#"
        INSERT INTO
            users  (email, name, password)
            VALUES ($1, $2, $3)
        RETURNING id
    "#, email, username, password).fetch_one(pool()).await
        .map_err(RealWorldError::DB)?
        .id;

    let jwt_token = config::issue_jwt_for_user_of_id(new_user_id);

    Ok(Created(UserResponse {
        user: User {
            email:    email.into(),
            token:    jwt_token,
            username: username.into(),
            bio:      None,
            image:    None,
        },
    }))
}
