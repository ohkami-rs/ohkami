use std::borrow::Cow;
use ohkami::{Ohkami, Route, utils::{Payload, Deserialize, Deserializer}, typed::{Created, OK}};
use crate::{
    models::{User, UserResponse},
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

#[Payload(JSON)]
struct LoginRequest<'req> {
    user: LoginRequestUser<'req>,
} const _: () = {
    impl<'req> Deserialize<'req> for LoginRequest<'req> {
        fn deserialize<D: Deserializer<'req>>(deserializer: D) -> Result<Self, D::Error> {
            Ok(Self {
                user: LoginRequestUser::deserialize(deserializer)?,
            })
        }
    }
};
#[derive(Deserialize)]
struct LoginRequestUser<'req> {
    email:    &'req str,
    password: &'req str,
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

    Ok(OK(UserResponse {
        user: User {
            email: u.email,
            jwt:   config::issue_jwt_for_user_of_id(u.id),
            name:  u.name,
            bio:   u.bio,
            image: u.image_url,
        },
    }))
}

#[Payload(JSOND)]
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
                u.email = $1
        )
    "#, email).fetch_one(pool()).await
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
